//
// Created by chiel on 28-12-19.
//

// #include "Connection.h"
// #include <aawireless/log/Log.h>
// #include <ControlMessageIdsEnum.pb.h>
// #include <AuthCompleteIndicationMessage.pb.h>

Connection(boost::asio::io_context &ioService,
            std::shared_ptr<f1x::aasdk::messenger::ICryptor> cryptor,
            std::shared_ptr<f1x::aasdk::transport::ITransport> transport,
            std::shared_ptr<f1x::aasdk::messenger::IMessageInStream> inStream,
            std::shared_ptr<f1x::aasdk::messenger::IMessageOutStream> outStream) ->
        receiveStrand(ioService),
        sendStrand(ioService),
        cryptor(std::move(cryptor)),
        transport(std::move(transport)),
        inStream(std::move(inStream)),
        outStream(std::move(outStream)) {
}

pub fn start() {
    cryptor->init();
    active = true;
}

pub fn stop() {
    receiveStrand.dispatch([this, self = this->shared_from_this()]() {
        AW_LOG(info) << "[AndroidAutoEntity] stop.";
        active = false;

        try {
            transport->stop();
            cryptor->deinit();
        } catch (...) {
            AW_LOG(error) << "[AndroidAutoEntity] exception in stop.";
        }
    });
}

pub fn receive(promise: f1x::aasdk::messenger::ReceivePromise::Pointer) {
    receiveStrand.dispatch([this, self = this->shared_from_this(), promise]() {
        if (active) {
            auto innerPromise = f1x::aasdk::messenger::ReceivePromise::defer(receiveStrand);
            innerPromise->then(
                    std::bind(&handleMessage, this->shared_from_this(), std::placeholders::_1, promise),
                    std::bind(&f1x::aasdk::messenger::ReceivePromise::reject, promise, std::placeholders::_1));

            inStream->startReceive(std::move(innerPromise));
        }
    });
}

pub fn send(message: f1x::aasdk::messenger::Message::Pointer, promise: f1x::aasdk::messenger::SendPromise::Pointer) {
    outStream->stream(std::move(message), std::move(promise));
}

pub fn onHandshake(&payload: f1x::aasdk::common::DataConstBuffer, promise: f1x::aasdk::io::Promise<void>::Pointer) {
    AW_LOG(info) << "Handshake, size: " << payload.size;

    try {
        cryptor->writeHandshakeBuffer(payload);

        auto message(std::make_shared<f1x::aasdk::messenger::Message>(
                f1x::aasdk::messenger::ChannelId::CONTROL,
                f1x::aasdk::messenger::EncryptionType::PLAIN,
                f1x::aasdk::messenger::MessageType::SPECIFIC));

        if (!cryptor->doHandshake()) {
            AW_LOG(info) << "Continue handshake.";
            message->insertPayload(f1x::aasdk::messenger::MessageId(
                    f1x::aasdk::proto::ids::ControlMessage::SSL_HANDSHAKE).getData());
            message->insertPayload(cryptor->readHandshakeBuffer());
        } else {
            AW_LOG(info) << "Auth completed.";
            message->insertPayload(f1x::aasdk::messenger::MessageId(
                    f1x::aasdk::proto::ids::ControlMessage::AUTH_COMPLETE).getData());
            f1x::aasdk::proto::messages::AuthCompleteIndication authCompleteIndication;
            authCompleteIndication.set_status(f1x::aasdk::proto::enums::Status::OK);
            message->insertPayload(authCompleteIndication);
        }

        this->send(std::move(message), std::move(promise));
    }
    catch (&e: f1x::aasdk::error::Error) {
        AW_LOG(error) << "Handshake error: " << e.what();
        promise->reject(e);
    }
}

pub fn handleMessage(message: f1x::aasdk::messenger::Message::Pointer, promise: f1x::aasdk::messenger::ReceivePromise::Pointer) {
    if (message->getChannelId() == f1x::aasdk::messenger::ChannelId::CONTROL) {
        f1x::aasdk::messenger::MessageId messageId(message->getPayload());
        f1x::aasdk::common::DataConstBuffer payload(message->getPayload(),
                                                    messageId.getSizeOf());

        if (messageId.getId() == f1x::aasdk::proto::ids::ControlMessage::SSL_HANDSHAKE) {
            auto innerPromise = f1x::aasdk::io::Promise<void>::defer(receiveStrand);
            innerPromise->then(
                    [this, self = this->shared_from_this(), promise]() {
                        inStream->startReceive(std::move(promise));
                    },
                    std::bind(&f1x::aasdk::messenger::ReceivePromise::reject, promise, std::placeholders::_1)
            );
            onHandshake(payload, std::move(innerPromise));
            return;
        }
    }
    promise->resolve(message);
}
