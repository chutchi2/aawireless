// #include "Connection.h"
// #include <aawireless/log/Log.h>
// #include <ControlMessageIdsEnum.pb.h>
// #include <AuthCompleteIndicationMessage.pb.h>
use std;
use boost;
use f1x;

pub struct Connection {
    receiveStrand: boost::asio::io_service::strand,
    sendStrand: boost::asio::io_service::strand,
    cryptor: Box<f1x::aasdk::messenger::ICryptor>,
    transport: Box<f1x::aasdk::transport::ITransport>,
    inStream: Box<f1x::aasdk::messenger::IMessageInStream>,
    outStream: Box<f1x::aasdk::messenger::IMessageOutStream>,
    active: bool,
}
impl Connection {
    pub fn new(&self,
        ioService: &boost::asio::io_context,
        cryptor: Box<f1x::aasdk::messenger::ICryptor>,
        transport: Box<f1x::aasdk::transport::ITransport>,
        inStream: Box<f1x::aasdk::messenger::IMessageInStream>,
        outStream: Box<f1x::aasdk::messenger::IMessageOutStream>,) -> Self {
            Self {
                receiveStrand: ioService,
                sendStrand: ioService,
                cryptor: std::move(cryptor),
                transport: std::move(transport),
                inStream: std::move(inStream),
                outStream: std::move(outStream),
                active: false
            }
    }
    
    pub fn start(&self) {
        self.cryptor.init();
        self.active = true;
    }
    
    pub fn stop(&self) {
        self.receiveStrand.dispatch([this, self = this.shared_from_this()]() {
            AW_LOG(info) << "[AndroidAutoEntity] stop.";
            self.active = false;
    
            try {
                transport.stop();
                cryptor.deinit();
            } catch (...) {
                AW_LOG(error) << "[AndroidAutoEntity] exception in stop.";
            }
        });
    }
    
    pub fn receive(&self, promise: f1x::aasdk::messenger::ReceivePromise::Pointer) {
        self.receiveStrand.dispatch([this, self = this.shared_from_this(), promise]() {
            if (self.active) {
                let innerPromise = f1x::aasdk::messenger::ReceivePromise::defer(self.receiveStrand);
                innerPromise.then(
                        std::bind(&self.handleMessage, this.shared_from_this(), std::placeholders::_1, promise),
                        std::bind(&f1x::aasdk::messenger::ReceivePromise::reject, promise, std::placeholders::_1));
    
                self.inStream.startReceive(std::move(innerPromise));
            }
        });
    }
    
    pub fn send(&self, message: f1x::aasdk::messenger::Message::Pointer, promise: f1x::aasdk::messenger::SendPromise::Pointer) {
        self.outStream.stream(std::move(message), std::move(promise));
    }
    
    pub fn onHandshake(&self, &payload: f1x::aasdk::common::DataConstBuffer, promise: f1x::aasdk::io::Promise<void>::Pointer) {
        AW_LOG(info) << "Handshake, size: " << payload.size;
    
        try {
            cryptor.writeHandshakeBuffer(payload);
    
            let message: auto = (std::make_shared<f1x::aasdk::messenger::Message>(
                    f1x::aasdk::messenger::ChannelId::CONTROL,
                    f1x::aasdk::messenger::EncryptionType::PLAIN,
                    f1x::aasdk::messenger::MessageType::SPECIFIC));
    
            if (!cryptor.doHandshake()) {
                AW_LOG(info) << "Continue handshake.";
                message.insertPayload(f1x::aasdk::messenger::MessageId(f1x::aasdk::proto::ids::ControlMessage::SSL_HANDSHAKE).getData());
                message.insertPayload(cryptor.readHandshakeBuffer());
            } else {
                AW_LOG(info) << "Auth completed.";
                message.insertPayload(f1x::aasdk::messenger::MessageId(f1x::aasdk::proto::ids::ControlMessage::AUTH_COMPLETE).getData());
                let authCompleteIndication: f1x::aasdk::proto::messages::AuthCompleteIndication;
                authCompleteIndication.set_status(f1x::aasdk::proto::enums::Status::OK);
                message.insertPayload(authCompleteIndication);
            }
    
            self.send(std::move(message), std::move(promise));
        }
        catch (e: &f1x::aasdk::error::Error) {
            AW_LOG(error) << "Handshake error: " << e.what();
            promise.reject(e);
        }
    }
    
    pub fn handleMessage(&self, message: f1x::aasdk::messenger::Message::Pointer, promise: f1x::aasdk::messenger::ReceivePromise::Pointer) {
        if (message.getChannelId() == f1x::aasdk::messenger::ChannelId::CONTROL) {
            let messageId = f1x::aasdk::messenger::MessageId(message.getPayload());
            let payload = f1x::aasdk::common::DataConstBuffer(message.getPayload(),
                                                        messageId.getSizeOf());
    
            if (messageId.getId() == f1x::aasdk::proto::ids::ControlMessage::SSL_HANDSHAKE) {
                let innerPromise = f1x::aasdk::io::Promise<void>::defer(receiveStrand);
                innerPromise.then(
                        [this, self = this.shared_from_this(), promise]() {
                            self.inStream.startReceive(std::move(promise));
                        },
                        std::bind(&f1x::aasdk::messenger::ReceivePromise::reject, promise, std::placeholders::_1)
                );
                self.onHandshake(payload, std::move(innerPromise));
                return;
            }
        }
        promise.resolve(message);
    }
}
