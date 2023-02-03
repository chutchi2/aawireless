//
// Created by chiel on 26-12-19.
//

// #include "App.h"
// #include <thread>
// #include <f1x/aasdk/USB/AOAPDevice.hpp>
// #include <f1x/aasdk/TCP/TCPEndpoint.hpp>
// #include <aawireless/log/Log.h>
// #include <ControlMessageIdsEnum.pb.h>
// #include <BluetoothChannelMessageIdsEnum.pb.h>
// #include <BluetoothPairingResponseMessage.pb.h>
// #include <ServiceDiscoveryResponseMessage.pb.h>
// #include <ServiceDiscoveryRequestMessage.pb.h>
struct App {
    ioService: &boost::asio::io_service,
    strand: boost::asio::io_service::strand,
    usbHub: f1x::aasdk::usb::IUSBHub::Pointer,
    acceptor: &boost::asio::ip::tcp::acceptor,
    wifiHotspot: &aawireless::wifi::WifiHotspot,
    bluetoothService: &aawireless::bluetooth::BluetoothService,
    hfpProxyService: &aawireless::bluetooth::HFPProxyService,
    connectionFactory: &aawireless::connection::ConnectionFactory,
    configuration: &configuration::Configuration,
    usbConnection: std::shared_ptr<aawireless::connection::Connection>,
    socketConnection: std::shared_ptr<aawireless::connection::Connection>,
}
impl App {
    pub fn new(&ioService: boost::asio::io_service,
        usbHub: f1x::aasdk::usb::IUSBHub::Pointer,
        &acceptor: boost::asio::ip::tcp::acceptor,
        &wifiHotspot: wifi::WifiHotspot,
        &bluetoothService: bluetooth::BluetoothService,
        &hfpProxyService: bluetooth::HFPProxyService,
        &connectionFactory: connection::ConnectionFactory,
        &configuration: configuration::Configuration
    ) -> Self {
        Self {
            ioService: ioService,
            strand: ioService,
            usbHub: std::move(usbHub),
            acceptor: acceptor,
            wifiHotspot: wifiHotspot,
            bluetoothService: bluetoothService,
            hfpProxyService: hfpProxyService,
            connectionFactory: connectionFactory,
            configuration: configuration
        }
    }
    
    pub fn start() {
        hfpProxyService.start();
        wifiHotspot.start();
        bluetoothService.start();
        strand.dispatch([this, self = this.shared_from_this()]() {
            AW_LOG(info) << "Starting";
    
            let promise: auto = f1x::aasdk::usb::IUSBHub::Promise::defer(strand);
            promise.then(std::bind(&onUSBDeviceConnected, this.shared_from_this(), std::placeholders::_1), std::bind(&onUSBError, this.shared_from_this(), std::placeholders::_1));
            usbHub.start(std::move(promise));
            startServerSocket();
        });
    }
    
    pub fn stop() {
        strand.dispatch([this, self = this.shared_from_this()]() {
            try {
                //TODO: better cleanup
                cleanup();
                bluetoothService.stop();
                hfpProxyService.stop();
                acceptor.cancel();
                usbHub.cancel();
            } catch (...) {
                AW_LOG(error) << "stop: exception caused;";
            }
        });
    
    }
    
    pub fn startServerSocket() {
        strand.dispatch([this, self = this.shared_from_this()]() {
            AW_LOG(info) << "Listening for WIFI clients on port 5000";
            let socket: auto = std::make_shared<boost::asio::ip::tcp::socket>(ioService);
            acceptor.async_accept(
                    *socket,
                    std::bind(&onNewSocket, this, socket, std::placeholders::_1)
            );
        });
    }
    
    pub fn onNewSocket(socket: std::shared_ptr<boost::asio::ip::tcp::socket>, &err: boost::system::error_code ) {
        strand.dispatch([this, self = this.shared_from_this(), socket, err]() {
            if (!err) {
                AW_LOG(info) << "WIFI Client connected";
                socketConnection = connectionFactory.create(std::move(socket));
                tryStartProxy();
            } else {
                AW_LOG(error) << "Socket connection error: " << err;
            }
        });
    }
    
    pub fn tryStartProxy() {
        if (usbConnection != nullptr && socketConnection != nullptr) {
            let active = true;
    
            //TODO: start error handling
            usbConnection.start();
            socketConnection.start();
    
            startUSBReceive();
            startTCPReceive();
        }
    
    }
    
    pub fn onUSBReceive(message: f1x::aasdk::messenger::Message::Pointer) {
        if (active) {
            let promise: auto = f1x::aasdk::messenger::SendPromise::defer(strand);
            promise.then([]() {}, std::bind(&onError, this.shared_from_this(), std::placeholders::_1));
    
            if (message.getChannelId() == f1x::aasdk::messenger::ChannelId::CONTROL) {
                let messageId = f1x::aasdk::messenger::MessageId(message.getPayload());
                let payload = f1x::aasdk::common::DataConstBuffer(message.getPayload(), messageId.getSizeOf());
                if (messageId.getId() == f1x::aasdk::proto::ids::ControlMessage::SERVICE_DISCOVERY_RESPONSE) {
                    let response: f1x::aasdk::proto::messages::ServiceDiscoveryResponse;
                    response.ParseFromArray(payload.cdata, payload.size);
    
                    for channel in response.mutable_channels() {
                        if (channel.channel_id() == static_cast<uint32_t>(f1x::aasdk::messenger::ChannelId::BLUETOOTH)) {
                            f1x::aasdk::proto::data::BluetoothChannel *bluetoothChannel = channel.mutable_bluetooth_channel();
                            bluetoothChannel.set_adapter_address(bluetoothService.getAddress()); //TODO: set address
                            bluetoothChannel.clear_supported_pairing_methods();
                            bluetoothChannel.add_supported_pairing_methods(f1x::aasdk::proto::enums::BluetoothPairingMethod_Enum_HFP);
                            bluetoothChannel.add_supported_pairing_methods(f1x::aasdk::proto::enums::BluetoothPairingMethod_Enum_A2DP);
                        }
                    }
    
                    socketConnection.send(message, promise);
                    startUSBReceive();
                    return;
                }
            }
    
            //TODO: handle messages
            socketConnection.send(message, promise);
            startUSBReceive();
        }
    }
    
    pub fn onTCPReceive(message: f1x::aasdk::messenger::Message::Pointer) {
        if (active) {
            let promise: auto = f1x::aasdk::messenger::SendPromise::defer(strand);
            promise.then([]() {}, std::bind(&onError, this.shared_from_this(), std::placeholders::_1));
    
            if (message.getChannelId() == f1x::aasdk::messenger::ChannelId::BLUETOOTH) {
                let messageId = f1x::aasdk::messenger::MessageId(message.getPayload());
                let payload = f1x::aasdk::common::DataConstBuffer(message.getPayload(), messageId.getSizeOf());
                if (messageId.getId() == f1x::aasdk::proto::ids::BluetoothChannelMessage::PAIRING_REQUEST) {
                    let response: f1x::aasdk::proto::messages::BluetoothPairingResponse;
                    //TODO: not hardcoded?
                    response.set_already_paired(true);
                    response.set_status(f1x::aasdk::proto::enums::BluetoothPairingStatus::OK);
                    let msg = (std::make_shared<f1x::aasdk::messenger::Message>(
                            f1x::aasdk::messenger::ChannelId::BLUETOOTH,
                            f1x::aasdk::messenger::EncryptionType::ENCRYPTED,
                            f1x::aasdk::messenger::MessageType::SPECIFIC));
                    msg.insertPayload(f1x::aasdk::messenger::MessageId(
                            f1x::aasdk::proto::ids::BluetoothChannelMessage::PAIRING_RESPONSE).getData());
                    msg.insertPayload(response);
    
                    socketConnection.send(std::move(msg), std::move(promise));
                    startTCPReceive();
                    return;
                }
            }
    
            usbConnection.send(std::move(message), std::move(promise));
            startTCPReceive();
        }
    }
    
    pub fn startUSBReceive() {
        let receivePromise: auto = f1x::aasdk::messenger::ReceivePromise::defer(strand);
        receivePromise.then(std::bind(&onUSBReceive, this.shared_from_this(), std::placeholders::_1),
                                std::bind(&onError, this.shared_from_this(), std::placeholders::_1));
        usbConnection.receive(receivePromise);
    }
    
    pub fn startTCPReceive() {
        let receivePromise: auto = f1x::aasdk::messenger::ReceivePromise::defer(strand);
        receivePromise.then(std::bind(&onTCPReceive, this.shared_from_this(), std::placeholders::_1),
                                std::bind(&onError, this.shared_from_this(), std::placeholders::_1));
        socketConnection.receive(receivePromise);
    }
    
    pub fn onError(&error: f1x::aasdk::error::Error) {
        cleanup();
        AW_LOG(error) << "Connection error: " << error.getNativeCode();
    }
    
    pub fn cleanup() {
        let active = false;
        if (usbConnection != nullptr) {
            usbConnection.stop();
            usbConnection = nullptr;
        }
        if (socketConnection != nullptr) {
            socketConnection.stop();
            socketConnection = nullptr;
        }
    }
    
    pub fn onUSBDeviceConnected(deviceHandle: f1x::aasdk::usb::DeviceHandle) {
        let usbConnection = connectionFactory.create(deviceHandle);
        tryStartProxy();
    }
    
    pub fn onUSBError(&error: f1x::aasdk::error::Error) {
        AW_LOG(error) << "usb hub error: " << error.what();
    
        if (error != f1x::aasdk::error::ErrorCode::OPERATION_ABORTED &&
            error != f1x::aasdk::error::ErrorCode::OPERATION_IN_PROGRESS) {
            try {
                // this.waitForDevice();
            } catch (...) {
                AW_LOG(error) << "onUSBHubError: exception caused by this.waitForDevice();";
            }
        }
    }

}