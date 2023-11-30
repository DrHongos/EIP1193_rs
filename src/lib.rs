use js_sys::Function;
use wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};
use js_sys::Promise;
use std::vec::Vec;
use web_sys::Event;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::any::Any;

pub type Callback = Box<dyn Fn(Result<JsValue, JsValue>) -> ()>;

#[derive(Debug, Clone)]
pub struct Provider {
    pub this: JsValue,
//    pub is_metamask: bool,  // Property
    pub request: Function,  // Method
    pub on: Function,       // Events
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMethod {
    method: String,
    params: Option<Vec<String>>,
}

#[wasm_bindgen]
extern "C" {
   #[wasm_bindgen(js_namespace=["console"])]
    pub fn log(value: &str);
}

impl Provider {

    pub fn new() -> Self {
        let provider = web_sys::window().unwrap().get("ethereum").unwrap();
        /* 
        // BUG: not working
        let is_metamask_js = js_sys::Reflect::get(
            &provider,
            &JsValue::from("isMetamask")
        ).unwrap();
        let is_metamask: bool = serde_wasm_bindgen::from_value(is_metamask_js).unwrap_or(false);
         */
        let request = js_sys::Reflect::get(
            &provider,
            &JsValue::from("request")
        ).unwrap();
        let on = js_sys::Reflect::get(
            &provider,
            &JsValue::from("on")
        ).unwrap();
        return Provider {
            this: JsValue::from(provider),
//            is_metamask: is_metamask, 
            request: Function::from(request),
            on: Function::from(on),
        };
    }

    pub fn on(self, event: String, callback: Box<dyn FnMut(Event)>) -> () {
        // doc: https://rustwasm.github.io/wasm-bindgen/examples/paint.html
        let closure = Closure::wrap(callback);        
        spawn_local(async move {
            let _l = self.set_listener_and_callback(
                event, 
                closure.as_ref().unchecked_ref::<Function>().to_owned()
            ).await.expect("Could not set listener");
            closure.forget();
        })
    }

    pub async fn set_listener_and_callback(self, event: String, callback: Function) -> Result<JsValue, JsValue> {
        match self.on.call2(
            &self.this, 
            //js_sys::Array::from(JsValue::from(event).as_ref()).as_ref()
            &serde_wasm_bindgen::to_value(&event).unwrap().into(), 
            &callback,
        ) {
            Ok(r) => {
                let p = Promise::resolve(&r.into());
                Ok(wasm_bindgen_futures::JsFuture::from(p).await?)
            },
            Err(e) => Err(e)
        }
    }

    pub async fn async_request(self, method: String, params: Option<Vec<String>> ) -> Result<JsValue, JsValue> {
        let args = RequestMethod{method, params};
        //log(format!("{:#?}", args).as_str());
        let ret = self.request.call1(
            &self.this,
            &serde_wasm_bindgen::to_value(&args).unwrap(),
        );
        match ret {
            Ok(s)=> {
                let promise = Promise::resolve(&s.into());
                Ok(wasm_bindgen_futures::JsFuture::from(promise).await?)
            },
           Err(e) => Err(e)
        }
    }

    pub fn request(
        self,
        method: String,
        params: Option<Vec<String>>,
        ctx: Box<dyn Any>,
        callback: Box<dyn Fn(Result<JsValue, JsValue>, Box<dyn Any>) -> ()>
    ) -> () {
        spawn_local(async move {
            callback(self.async_request(method.clone(), params).await, ctx)
        });

    }
}
