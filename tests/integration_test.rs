use std::io::Cursor;
use payment_engine::App;
use tokio;

async fn process_csv_string(csv_content: &str) -> String {
    let mut output = Cursor::new(Vec::new());
    App::run(csv_content.as_bytes(), &mut output, true).await.unwrap();
    
    String::from_utf8(output.into_inner()).unwrap()
}

#[tokio::test]
async fn test_basic_transaction_flow() {
    let csv_content = "\
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,1.5,0,1.5,false
2,2.0,0,2.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_dispute_flow1() {
    // Create test CSV content with dispute scenario
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,20.0
dispute,1,1,
resolve,1,1,
deposit,1,3,50.0
dispute,1,1,
resolve,1,1,
resolve,1,1,
chargeback,1,1,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,130.0,0.0,130.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
} 

#[tokio::test]
async fn test_disputed_flow2() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,0.0,100.0,100.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_disputed_twice() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
dispute,1,1,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,0.0,100.0,100.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv); 
}

#[tokio::test]
async fn test_withdrawal_more_than_available() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,150.0";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,100.0,0,100.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_dispute_more_than_available() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,50.0
dispute,1,2,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,50.0,0,50.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_dispute_resolve() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
resolve,1,1,
";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,100.0,0.0,100.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_chargeback_without_dispute() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
chargeback,1,1,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,100.0,0,100.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_chargeback_with_dispute() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
chargeback,1,1,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,0.0,0.0,0.0,true
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}

#[tokio::test]
async fn test_chargeback_with_resolve() {
    let csv_content = "\
type,client,tx,amount
deposit,1,1,100.0
resolve,1,1,
chargeback,1,1,";

    let expected_accounts_csv = "\
client,available,held,total,locked
1,100.0,0,100.0,false
";

    assert_eq!(process_csv_string(csv_content).await, expected_accounts_csv);
}   