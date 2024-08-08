use std::collections::HashMap;

use anyhow::Result;

use rquickjs::{Context, FromJs, Function, IntoJs, Object, Promise, Runtime};
use typed_builder::TypedBuilder;

#[allow(unused)]
pub struct JsWorker {
    rt: Runtime,
    ctx: Context,
}

#[derive(Debug, TypedBuilder)]
pub struct Request {
    pub headers: HashMap<String, String>,
    #[builder(setter(into))]
    pub method: String,
    #[builder(setter(into))]
    pub url: String,
    #[builder(default, setter(strip_option))]
    pub body: Option<String>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
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

    pub fn run(&self, name: &str, req: Request) -> Result<Response> {
        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let handlers: Object = global.get("handlers")?;
            let fun: Function = handlers.get(name)?;
            let v: Promise = fun.call((req,))?;

            Ok::<_, anyhow::Error>(v.finish()?)
        })
    }
}

impl<'js> IntoJs<'js> for Request {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;
        obj.set("headers", self.headers.into_js(ctx)?)?;
        obj.set("method", self.method.into_js(ctx)?)?;
        obj.set("url", self.url.into_js(ctx)?)?;
        obj.set("body", self.body.into_js(ctx)?)?;
        Ok(obj.into())
    }
}

impl<'js> FromJs<'js> for Response {
    fn from_js(_ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = value.into_object().unwrap();
        let status = obj.get("status")?;
        let headers = obj.get("headers")?;
        let body = obj.get("body")?;

        Ok(Response {
            status,
            headers,
            body,
        })
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

        let req = Request::builder()
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
