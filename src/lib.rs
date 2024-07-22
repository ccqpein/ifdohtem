use std::{
    collections::BTreeMap,
    io::{Error, ErrorKind},
    time::Duration,
};

use actix_web::rt::time;
use csv::WriterBuilder;
use serde_json::Value;

pub mod caller;
pub mod xml_parser;

pub struct Reports(
    pub BTreeMap<String, f64>,
    pub BTreeMap<String, f64>,
    pub Vec<Value>,
);

pub async fn payouts_call(
    rows: Vec<xml_parser::Row>,
) -> Result<Reports, Box<dyn std::error::Error>> {
    let mut count = 0;
    let mut interval = time::interval(Duration::from_secs(60));

    // Total amount of funds paid out per unique source account.
    let mut report1 = BTreeMap::new();
    // Total amount of funds paid out per Dunkin branch.
    let mut report2 = BTreeMap::new();
    // The status of every payment and its relevant metadata.
    let mut report3 = vec![];

    // generate all entity/account/etc.
    for row in &rows {
        let individual;
        if count < 600 {
            individual = caller::make_new_individual_entity(row).await?;
            count += 1;
        } else {
            interval.tick().await;
            count = 0;
            individual = caller::make_new_individual_entity(row).await?;
            count += 1;
        }

        let corporation;
        if count < 600 {
            corporation = caller::make_new_corporation_entity(row).await?;
            count += 1;
        } else {
            interval.tick().await;
            count = 0;
            corporation = caller::make_new_corporation_entity(row).await?;
            count += 1;
        }

        //
        let corp_account;
        if count < 600 {
            corp_account = caller::make_new_account_entity(
                row,
                corporation["id"].as_str().ok_or(Error::new(
                    ErrorKind::Other,
                    "cannot get the corporation id",
                ))?,
            )
            .await?;
            count += 1;
        } else {
            interval.tick().await;
            count = 0;
            corp_account = caller::make_new_account_entity(
                row,
                corporation["id"].as_str().ok_or(Error::new(
                    ErrorKind::Other,
                    "cannot get the corporation id",
                ))?,
            )
            .await?;
            count += 1;
        }

        let loan_account;
        if count < 600 {
            loan_account = caller::make_new_liability_entity(
                row,
                individual["id"]
                    .as_str()
                    .ok_or(Error::new(ErrorKind::Other, "cannot get the individual id"))?,
            )
            .await?;
            count += 1;
        } else {
            interval.tick().await;
            count = 0;
            loan_account = caller::make_new_liability_entity(
                row,
                individual["id"]
                    .as_str()
                    .ok_or(Error::new(ErrorKind::Other, "cannot get the individual id"))?,
            )
            .await?;
            count += 1;
        }

        // payment
        if count < 600 {
            match caller::make_new_payment_entity(
                row,
                corp_account["id"].as_str().ok_or(Error::new(
                    ErrorKind::Other,
                    "cannot get the corp_account id",
                ))?,
                loan_account["id"].as_str().ok_or(Error::new(
                    ErrorKind::Other,
                    "cannot get the loan_account id",
                ))?,
            )
            .await
            {
                Ok(resp) => {
                    let en = report1
                        .entry(corp_account["id"].as_str().unwrap().to_string())
                        .or_insert(0_f64);
                    *en += resp["amount"]
                        .as_f64()
                        .ok_or(Error::new(ErrorKind::Other, "number parsed failed"))
                        .unwrap();

                    let en = report2
                        .entry(row.employee.dunkin_branch.clone())
                        .or_insert(0_f64);
                    *en += resp["amount"]
                        .as_f64()
                        .ok_or(Error::new(ErrorKind::Other, "number parsed failed"))
                        .unwrap();

                    report3.push(resp);
                }
                Err(_) => continue,
            };
            count += 1;
        } else {
            interval.tick().await;
            count = 0;
            match caller::make_new_payment_entity(
                row,
                corp_account["id"].as_str().ok_or(Error::new(
                    ErrorKind::Other,
                    "cannot get the corp_account id",
                ))?,
                loan_account["id"].as_str().ok_or(Error::new(
                    ErrorKind::Other,
                    "cannot get the loan_account id",
                ))?,
            )
            .await
            {
                Ok(resp) => {
                    let en = report1
                        .entry(corp_account["id"].as_str().unwrap().to_string())
                        .or_insert(0_f64);
                    *en += resp["amount"]
                        .as_f64()
                        .ok_or(Error::new(ErrorKind::Other, "number parsed failed"))
                        .unwrap();

                    let en = report2
                        .entry(row.employee.dunkin_branch.clone())
                        .or_insert(0_f64);
                    *en += resp["amount"]
                        .as_f64()
                        .ok_or(Error::new(ErrorKind::Other, "number parsed failed"))
                        .unwrap();

                    report3.push(resp);
                }
                Err(_) => continue,
            };
            count += 1;
        }
    }

    Ok(Reports(report1, report2, report3))
}

pub fn save_btreemap_to_csv(
    path: &str,
    map: &BTreeMap<String, f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = WriterBuilder::new().from_path(path)?;
    for (key, value) in map {
        wtr.write_record(&[key, &value.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn save_vec_to_csv(path: &str, vec: &Vec<Value>) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = WriterBuilder::new().from_path(path)?;
    for value in vec {
        wtr.write_record(&[value.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}
