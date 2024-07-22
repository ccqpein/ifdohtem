#![doc = r"caller wrap all api call"]

use std::io::{Error, ErrorKind};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::xml_parser::Row;

// enum Entity {
//     Individual(IndividualEntity),
//     Corporation(CorporationEntity),
// }

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
        obj["corporation"]["owners"] = json!([{"phone": "15121231111",}]);

        serde_json::to_string(&obj)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct AccountEntity {
    #[serde(rename = "type")]
    account_type: String,
    abarouting: String,
    account_number: String,
}

impl AccountEntity {
    fn to_api_request_json(&self, holder_id: &str) -> Result<String, serde_json::Error> {
        let mut obj = serde_json::Map::new();
        obj.insert("holder_id".to_string(), holder_id.into());
        if self.abarouting == "" {
            obj.insert("liability".to_string(), json!(self));
        } else {
            obj.insert("ach".to_string(), json!(self));
        }

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

fn row_to_individual_entity(row: &Row) -> Result<IndividualEntity, Box<dyn std::error::Error>> {
    Ok(IndividualEntity {
        id: row.employee.dunkin_id.clone(),
        branch_id: row.employee.dunkin_branch.clone(),
        first_name: row.employee.first_name.clone(),
        last_name: row.employee.last_name.clone(),
        phone: "15121231111".to_string(),
        email: None,
        dob: row.employee.dob.clone(),
        address: None,
    })
}

fn row_to_corporation_entity(row: &Row) -> Result<CorporationEntity, Box<dyn std::error::Error>> {
    Ok(CorporationEntity {
        id: row.payor.dunkin_id.clone(),
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

fn row_to_account_entity(row: &Row) -> Result<AccountEntity, Box<dyn std::error::Error>> {
    Ok(AccountEntity {
        abarouting: row.payor.abarouting.clone(),
        account_number: row.payor.account_number.clone(),
        account_type: "checking".to_string(),
    })
}

fn row_to_liability_entity(row: &Row) -> Result<AccountEntity, Box<dyn std::error::Error>> {
    Ok(AccountEntity {
        abarouting: String::new(),
        account_number: row.payee.account_number.clone(),
        account_type: "loan".to_string(),
    })
}

fn row_to_amount(row: &Row) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(row.amount[1..].parse::<f64>()?)
}

const TOKEN: &'static str = include_str!("../data/methodfi-api");

pub async fn make_new_individual_entity(row: &Row) -> Result<Value, Box<dyn std::error::Error>> {
    let indvidual = row_to_individual_entity(row)?;
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

    Ok(json!(*response.text().await?))
}

pub async fn make_new_corporation_entity(row: &Row) -> Result<Value, Box<dyn std::error::Error>> {
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
        .json(&corporation.to_api_request_json()?)
        .send()
        .await?;
    Ok(json!(*response.text().await?))
}

pub async fn make_new_account_entity(
    row: &Row,
    holder_id: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let account = row_to_account_entity(row)?;
    let client = reqwest::Client::new();
    let response = client
        .post("https://production.methodfi.com/accounts")
        .header("Method-Version", "2024-04-04")
        .header(
            reqwest::header::AUTHORIZATION,
            "Bearer ".to_string() + TOKEN,
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&account.to_api_request_json(holder_id)?)
        .send()
        .await?;
    Ok(json!(*response.text().await?))
}

pub async fn make_new_liability_entity(
    row: &Row,
    holder_id: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let liability = row_to_liability_entity(row)?;
    let client = reqwest::Client::new();
    let response = client
        .post("https://production.methodfi.com/accounts")
        .header("Method-Version", "2024-04-04")
        .header(
            reqwest::header::AUTHORIZATION,
            "Bearer ".to_string() + TOKEN,
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&liability.to_api_request_json(holder_id)?)
        .send()
        .await?;
    Ok(json!(*response.text().await?))
}

pub async fn make_new_payment_entity(
    row: &Row,
    src: &str,
    target: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let amount = row_to_amount(row)?;
    let client = reqwest::Client::new();
    let response = client
        .post("https://production.methodfi.com/payments")
        .header("Method-Version", "2024-04-04")
        .header(
            reqwest::header::AUTHORIZATION,
            "Bearer ".to_string() + TOKEN,
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(
            json!(
                {"amount": amount,
                 "source": src,
                 "destination": target,
                 "description": ""}
            )
            .as_str()
            .ok_or(Error::new(
                ErrorKind::Other,
                "cannot get the corporation id",
            ))?,
        )
        .send()
        .await?;
    Ok(json!(*response.text().await?))
}

#[cfg(test)]
mod tests {
    use std::default;

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

        let v: AccountEntity = default::Default::default();
        dbg!(v.to_api_request_json(&uuid::Uuid::new_v4().to_string()));
    }
}
