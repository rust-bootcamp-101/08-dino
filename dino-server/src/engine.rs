use std::collections::HashMap;

use anyhow::Result;

use axum::{body::Body, response::Response};
use dino_macros::{FromJs, IntoJs};
use rquickjs::{Context, Function, Object, Promise, Runtime};
use typed_builder::TypedBuilder;

#[allow(unused)]
pub struct JsWorker {
    rt: Runtime,
    ctx: Context,
}

#[derive(Debug, TypedBuilder, IntoJs)]
pub struct Req<T> {
    #[builder(default)]
    pub query: HashMap<String, String>,
    #[builder(default)]
    pub params: HashMap<String, String>,
    #[builder(default)]
    pub headers: HashMap<String, String>,
    #[builder(setter(into))]
    pub method: String,
    #[builder(setter(into))]
    pub url: String,
    #[builder(default)]
    pub body: Option<T>,
}

#[derive(Debug, FromJs)]
pub struct Res<T> {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<T>,
}

// fn print(msg: String) {
//     println!("{msg}")
// }

impl JsWorker {
    pub fn try_new(module: &str) -> Result<Self> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        ctx.with(|ctx| {
            let global = ctx.globals();
            let ret: Object = ctx.eval(module)?;
            global.set("handlers", ret)?;
            // // setup print function
            // let fun = Function::new(ctx.clone(), print)?.with_name("print")?;
            // global.set("print", fun)?;
            Ok::<_, anyhow::Error>(())
        })?;

        Ok(Self { rt, ctx })
    }

    pub fn run<T>(&self, name: &str, req: Req<T>) -> Result<Res<T>>
    where
        T: for<'js> rquickjs::IntoJs<'js>,
        T: for<'js> rquickjs::FromJs<'js>,
    {
        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let handlers: Object = global.get("handlers")?;
            let fun: Function = handlers.get(name)?;
            let v: Promise = fun.call((req,))?;

            Ok::<_, anyhow::Error>(v.finish()?)
        })
    }
}

impl From<Res<String>> for Response {
    fn from(res: Res<String>) -> Self {
        let mut builder = Response::builder().status(res.status);
        for (k, v) in res.headers {
            builder = builder.header(k, v);
        }
        if let Some(body) = res.body {
            builder.body(body.into()).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_worker_should_work() -> Result<()> {
        let code = r#"
        (function(){
            async function hello(req){
                return {
                    status:200,
                    headers:{
                        "content-type":"application/json"
                    },
                    body: JSON.stringify(req)
                };
            }
            return{hello:hello};
        })()"#;

        let req: Req<String> = Req::builder()
            .method("GET")
            .url("https://example.com")
            .headers(HashMap::new())
            .build();
        let worker = JsWorker::try_new(code)?;
        let ret = worker.run("hello", req)?;
        assert_eq!(ret.status, 200);
        Ok(())
    }
}
