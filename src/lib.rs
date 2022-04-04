use std::collections::HashMap;

#[derive(Debug)]
pub enum IsoMessageTypes {
    Authorization,
    AuthorizationResponse,
    AuthorizationAdvice,
    AuthorizationAdviceResponse,
    Financial,
    FinancialResponse,
    FinancialAdvice,
    FinancialAdviceResponse,
    TokenManagement,
    TokenManagementResponse,
    Reversal,
    ReversalResponse,
    TokenNotification,
    TokenNotificationResponse,
    NetworkManagement,
    NetworkManagementResponse,
}

impl IsoMessageTypes {
    pub fn find_type(message_type: &str) -> Self {
        match message_type {
            "0100" => IsoMessageTypes::Authorization,
            "0110" => IsoMessageTypes::AuthorizationResponse,
            "0120" => IsoMessageTypes::AuthorizationAdvice,
            "0130" => IsoMessageTypes::AuthorizationAdviceResponse,
            "0200" => IsoMessageTypes::Financial,
            "0210" => IsoMessageTypes::FinancialResponse,
            "0220" => IsoMessageTypes::FinancialAdvice,
            "0230" => IsoMessageTypes::FinancialAdviceResponse,
            "0302" => IsoMessageTypes::TokenManagement,
            "0312" => IsoMessageTypes::TokenManagementResponse,
            "0420" => IsoMessageTypes::Reversal,
            "0430" => IsoMessageTypes::ReversalResponse,
            "0620" => IsoMessageTypes::TokenNotification,
            "0630" => IsoMessageTypes::TokenNotificationResponse,
            "0800" => IsoMessageTypes::NetworkManagement,
            "0810" => IsoMessageTypes::NetworkManagementResponse,
            _ => panic!("Unknown message type: {}", message_type),
        }
    }
    pub fn value(&self) -> &str {
        match *self {
            IsoMessageTypes::Authorization => "0100",
            IsoMessageTypes::AuthorizationResponse => "0110",
            IsoMessageTypes::AuthorizationAdvice => "0120",
            IsoMessageTypes::AuthorizationAdviceResponse => "0130",
            IsoMessageTypes::Financial => "0200",
            IsoMessageTypes::FinancialResponse => "0210",
            IsoMessageTypes::FinancialAdvice => "0220",
            IsoMessageTypes::FinancialAdviceResponse => "0230",
            IsoMessageTypes::TokenManagement => "0302",
            IsoMessageTypes::TokenManagementResponse => "0312",
            IsoMessageTypes::Reversal => "0420",
            IsoMessageTypes::ReversalResponse => "0430",
            IsoMessageTypes::TokenNotification => "0620",
            IsoMessageTypes::TokenNotificationResponse => "0630",
            IsoMessageTypes::NetworkManagement => "0800",
            IsoMessageTypes::NetworkManagementResponse => "0810",
        }
    }
}

pub struct IsoMessage {
    original_buffer: String,
    message: HashMap<String, String>,
}

impl IsoMessage {
    pub fn new(buffer: String) -> Self {
        let message = HashMap::new();

        IsoMessage {
            original_buffer: buffer,
            message,
        }
    }

    pub fn get_type(&self) -> IsoMessageTypes {
        let message_type = &self.original_buffer[..4];

        IsoMessageTypes::find_type(message_type)
    }
}
