use std::sync::{Arc, Mutex};
use std::time::Instant;

use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

use serde_json::json;

use crate::Robot;

use super::AppData;

pub struct WSHandler<R : Robot<N, D>, const N : usize, const D : usize> {
    pub data : Arc<Mutex<AppData<R, N, D>>>
}

impl<R : Robot<N, D> + 'static, const N : usize, const D : usize> Actor for WSHandler<R, N, D> {
    type Context = ws::WebsocketContext<Self>;
}

impl<R : Robot<N, D, Error = std::io::Error> + 'static, const N : usize, const D : usize> StreamHandler<Result<ws::Message, ws::ProtocolError>> for WSHandler<R, N, D> {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => { 
                let inst = Instant::now();

                let mut data = self.data.lock().unwrap();
                let results = data.intpr.interpret(&text, |_| {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, ""))
                });

                let mut results_json : Vec<serde_json::Value> = vec![];

                for res in results {
                    match res {
                        Ok(val) => results_json.push(val),
                        Err(err) => results_json.push(json![{
                            "code": err.kind() as u64,
                            "msg": err.to_string()
                        }])
                    }
                }

                let el_time = inst.elapsed().as_secs_f32();

                println!(" -> GCode: {} Lines executed in {}s", results_json.len(), el_time);
                
                ctx.text(serde_json::to_string(&json!({
                    "el": el_time,
                    "len": results_json.len(),
                    "res": results_json
                })).unwrap())
            },
            _ => ()
        }
    }
}