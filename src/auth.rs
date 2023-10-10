use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    Error,
};

use futures_util::future::LocalBoxFuture;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct BasicAuth;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for BasicAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = BasicAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BasicAuthMiddleware { service }))
    }
}

pub struct BasicAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for BasicAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());
        let mut authenticated = false;

        // Verificando se o header authorization foi mandando pelo usuario.
        // Se sim, vai decodificar o usuario e senha e ira setar a variavel
        // booleana authenticated caso a senha e usuario estiverem certo.
        if req.headers().iter().any(|x| x.0.to_string().eq("authorization"))
        {
            for header in req.headers() {
                if header.0.eq(&header::AUTHORIZATION) {
                    let credentials = header.1.as_bytes().split(|&x| x == 32).last().unwrap();
                    let credentials_decoded =
                        String::from_utf8(base64::decode(credentials).unwrap()).unwrap();
                    //senha hardcoded
                    authenticated = credentials_decoded.split(":").all(|x| x.eq("admin"));
                }
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            //Se não estiver autenticado, continuar mandando o header WWW_AUTHENTICATE
            //E enviar o status_code UNAUTHORIZED
            //Caso contrário, envia a resposta normal.
            if !authenticated {
                res.headers_mut().append(
                    header::WWW_AUTHENTICATE,
                    HeaderValue::from_str(r#"Basic realm="japabms""#).unwrap(),
                );
                *res.response_mut().status_mut() = StatusCode::UNAUTHORIZED;
            }

            Ok(res)
        })
    }
}
