use quick_xml::DeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Row {
    pub employee: Employee,
    pub payor: Payor,
    pub payee: Payee,
    pub amount: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Employee {
    pub dunkin_id: String,
    pub dunkin_branch: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(rename = "DOB")]
    pub dob: String,
    pub phone_number: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payor {
    pub dunkin_id: String,
    #[serde(rename = "ABARouting")]
    pub abarouting: String,
    pub account_number: String,
    pub name: String,
    #[serde(rename = "DBA")]
    pub dba: String,
    #[serde(rename = "EIN")]
    pub ein: String,
    pub address: Address,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Address {
    pub line1: String,
    pub city: String,
    pub state: String,
    pub zip: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payee {
    pub plaid_id: String,
    #[serde(rename = "LoanAccountNumber")]
    pub account_number: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "root")]
pub struct Root {
    pub row: Vec<Row>,
}

impl Root {
    pub fn to_string(&self) -> Result<String, DeError> {
        quick_xml::se::to_string(self)
    }
}

fn parse_row(xml: &str) -> Result<Row, DeError> {
    quick_xml::de::from_str(xml)
}

pub fn parse_xml(xml: &str) -> Result<Root, DeError> {
    quick_xml::de::from_str(xml)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::parse_xml;

    #[test]
    fn test_xml_parser() {
        let mut buf = String::new();
        File::open("data/onerow.xml")
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        dbg!(parse_xml(&buf));

        //let x = parse_xml(&buf).unwrap();
        //assert_eq!(x.to_string().unwrap(), buf)
    }

    // #[test]
    // fn test_row_to_csv() {
    //     let mut buf = String::new();
    //     File::open("data/onerow.xml")
    //         .unwrap()
    //         .read_to_string(&mut buf)
    //         .unwrap();

    //     let root = parse_xml(&buf).unwrap();
    //     dbg!(root.row[0].to_csv());
    // }
}
