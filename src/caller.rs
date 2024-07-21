#![doc = r"caller wrap all api call"]

use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::yml_parser::Row;

enum Entity {
    Individual(IndividualEntity),
    Corporation(CorporationEntity),
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct IndividualEntity {
    id: String,
    branch_id: String,
    first_name: String,
    last_name: String,
    phone: String,
    email: Option<String>,
    dob: String,
    #[serde(skip_serializing)]
    address: Option<Address>,
}

impl IndividualEntity {
    fn to_api_request_json(&self) -> Result<String, serde_json::Error> {
        let mut obj = serde_json::Map::new();
        obj.insert("type".to_string(), "individual".into());
        obj.insert("individual".to_string(), json!(self));
        obj.insert("address".to_string(), json!(self.address));
        serde_json::to_string(&obj)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct CorporationEntity {
    id: String,
    abarouting: String,
    account_number: String,
    name: String,
    dba: String,
    ein: String,
    owners: Option<Vec<IndividualEntity>>,
    #[serde(skip_serializing)]
    address: Address,
}

impl CorporationEntity {
    fn to_api_request_json(&self) -> Result<String, serde_json::Error> {
        let mut obj = serde_json::Map::new();
        obj.insert("type".to_string(), "corporation".into());
        obj.insert("corporation".to_string(), json!(self));
        obj.insert("address".to_string(), json!(self.address));
        serde_json::to_string(&obj)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Address {
    line1: String,
    line2: String,
    city: String,
    state: String,
    zip: String,
}

fn row_to_individual_entity(row: &Row) -> Result<IndividualEntity, Box<dyn Error>> {
    Ok(IndividualEntity {
        id: row.employee.dunkin_id.clone(),
        branch_id: row.employee.dunkin_branch.clone(),
        first_name: row.employee.first_name.clone(),
        last_name: row.employee.last_name.clone(),
        phone: row.employee.phone_number.clone(),
        email: None,
        dob: row.employee.dob.clone(),
        address: None,
    })
}

fn row_to_corporation_entity(row: &Row) -> Result<CorporationEntity, Box<dyn Error>> {
    Ok(CorporationEntity {
        id: row.payor.dunkin_id.clone(),
        abarouting: row.payor.abarouting.clone(),
        account_number: row.payor.account_number.clone(),
        name: row.payor.name.clone(),
        dba: row.payor.dba.clone(),
        ein: row.payor.ein.clone(),
        owners: None,
        address: Address {
            line1: row.payor.address.line1.clone(),
            line2: String::new(), //:= line2 not in xml
            city: row.payor.address.city.clone(),
            state: row.payor.address.state.clone(),
            zip: row.payor.address.zip.clone(),
        },
    })
}

const TOKEN: &'static str = include_str!("../data/methodfi-api");

async fn make_new_entity(row: &Row) -> Result<(), Box<dyn std::error::Error>> {
    let indvidual = row_to_individual_entity(row)?;
    let corporation = row_to_corporation_entity(row)?;

    let client = reqwest::Client::new();

    let response = client
        .post("https://production.methodfi.com/entities")
        .header("Method-Version", "2024-04-04")
        .header(
            reqwest::header::AUTHORIZATION,
            "Bearer ".to_string() + TOKEN,
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&indvidual.to_api_request_json()?)
        .send()
        .await?;

    let response = client
        .post("https://production.methodfi.com/entities")
        .header("Method-Version", "2024-04-04")
        .header(
            reqwest::header::AUTHORIZATION,
            "Bearer ".to_string() + TOKEN,
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&corporation.to_api_request_json()?)
        .send()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::default;

    use serde_json::json;

    use super::*;

    #[test]
    fn test_json_maker() {
        let v: CorporationEntity = default::Default::default();

        //dbg!(serde_json::to_value(&v));
        // let mut obj = serde_json::Map::new();
        // obj.insert("type".to_string(), "corporation".into());
        // obj.insert("corporation".to_string(), json!(v));
        //obj.insert("address".to_string(), json!(v));
        //dbg!(obj);

        //dbg!(json!({"type": "corporation", "corporation": serde_json::to_string(&v)}))
        dbg!(v.to_api_request_json());
    }
}
