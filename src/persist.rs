extern crate postgres;
extern crate chrono;

use postgres::{Connection, TlsMode};

use crate::domain::{Event,TestDto,TestDtoB};

static INSERT_EVENT: &str = "INSERT INTO events (aggregateid, sequence, time, payloadtype, payload, metadata)
                               VALUES ($1, $2, $3, $4, $5, $6)";
static SELECT_EVENTS: &str = "SELECT aggregateid, sequence, time, payloadtype, payload, metadata
                                FROM events
                                WHERE aggregateid = $1 ORDER BY sequence";

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    use serde_json::Value;
    use crate::domain::ProjectAggregate;
    use std::time::{Instant, SystemTime};
    use super::chrono::Utc;


    #[test]
    fn test_ok() {
        let mut object_map = HashMap::<&str,fn()->Box<dyn Event<ProjectAggregate>>>::new();
        object_map.insert("TestDto", || Box::new(TestDto::default()));
        object_map.insert("TestDtoB", || Box::new(TestDtoB::default()));

        let aggregate_id = format!("TST-{}", Uuid::new_v4().to_string());
        let events: Vec<Box<dyn Event<ProjectAggregate>>> = vec![
            Box::new(TestDto{
                id: aggregate_id.to_string(),
                full_name: "John Doe".to_string()
            }),
            Box::new(TestDtoB{
                id: aggregate_id.to_string(),
                email: "sample@example.com".to_string()
            })
        ];

        let conn = Connection::connect("postgresql://stc_user:stc_pass@localhost:5432/stc", TlsMode::None)
            .unwrap();

        for (i, event) in events.iter().enumerate() {
            let placeholder = "{}";
            match serde_json::to_value(event.as_ref()) {
                Ok(ser) => {
                    let sequence = i as i32;
                    let datetime = Utc::now().to_rfc2822();
                    let payload_type = event.name();
                    match conn.execute(INSERT_EVENT, &[&aggregate_id, &sequence, &placeholder, &payload_type, &ser, &ser]) {
                        Ok(result) => print!("insert result: {}\n", result),
                        Err(e) => print!("{}\n", e),
                    };
                },
                Err(_) => {},
            };
            // println!("{}",ser);
        }
        match conn.query(SELECT_EVENTS, &[&aggregate_id]) {
            Ok(rows) => {
                for (_i,row) in rows.iter().enumerate() {
                    let payload_type : String = row.get("payloadtype");
                    let payload : Value = row.get("payload");
                    let mut event = object_map.get(payload_type.as_str()).unwrap()();
                    event.from_json(payload);
                    print!("{:?}\n", event);
                }
            },
            Err(e) => print!("{}\n", e)
        }
    }
}