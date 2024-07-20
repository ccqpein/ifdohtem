use quick_xml::DeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Row {
    employee: Employee,
    payor: Payor,
    payee: Payee,
    amount: String,
}

// impl Row {
//     fn to_csv(&self) -> Result<String, Box<dyn std::error::Error>> {
//         let mut wtr = csv::Writer::from_writer(Vec::new());
//         wtr.serialize(self)?;
//         let data = String::from_utf8(wtr.into_inner()?)?;
//         Ok(data)
//     }
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Employee {
    dunkin_id: String,
    dunkin_branch: String,
    first_name: String,
    last_name: String,
    #[serde(rename = "DOB")]
    dob: String,
    phone_number: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payor {
    dunkin_id: String,
    #[serde(rename = "ABARouting")]
    abarouting: String,
    account_number: String,
    name: String,
    #[serde(rename = "DBA")]
    dba: String,
    #[serde(rename = "EIN")]
    ein: String,
    address: Address,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Address {
    line1: String,
    city: String,
    state: String,
    zip: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payee {
    plaid_id: String,
    loan_account_number: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "root")]
pub struct Root {
    row: Vec<Row>,
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
