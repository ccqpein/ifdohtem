use super::yml_parser::*;

/// flatten notification of all payments before the confirm
struct Notification {
    payer_id: String,
    pay_to_first_name: String,
    pay_to_last_name: String,
    amout: String,
}

/// Total amount of funds paid out per unique source account.
struct TotalAmountOfPerSourceAcc {}

/// Total amount of funds paid out per Dunkin branch.
struct TotalAmountOfPerBrach {}

/// The status of every payment and its relevant metadata.
struct Status {}
