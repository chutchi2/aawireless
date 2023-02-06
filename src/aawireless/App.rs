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

use std::{ptr};
use boost;
use f1x;
use crate::aawireless::bluetooth::*;
use crate::aawireless::connection::*;
use crate::aawireless::configuration::*;
use crate::aawireless::wifi::*;

pub struct App {
    ioService: &boost::asio::io_service,
    strand: boost::asio::io_service::strand,
    usbHub: f1x::aasdk::usb::IUSBHub::Pointer,
    acceptor: &boost::asio::ip::tcp::acceptor,
    wifiHotspot: &WifiHotspot::WifiHotspot,
    bluetoothService: BluetoothService::BluetoothService,
    hfpProxyService: HFPProxyService::HFPProxyService,
    connectionFactory: &ConnectionFactory::ConnectionFactory,
    configuration: &Configuration::Configuration,
    usbConnection: std::shared_ptr<Connection::Connection>,
    socketConnection: std::shared_ptr<Connection::Connection>,
    active: bool,
}
impl App {
    pub fn new(&ioService: boost::asio::io_service,
        usbHub: f1x::aasdk::usb::IUSBHub::Pointer,
        &acceptor: boost::asio::ip::tcp::acceptor,
        &wifiHotspot: WifiHotspot::WifiHotspot,
        &bluetoothService: BluetoothService::BluetoothService,
        &hfpProxyService: HFPProxyService::HFPProxyService,
        &connectionFactory: ConnectionFactory::ConnectionFactory,
        &configuration: Configuration::Configuration
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
            configuration: configuration,
            usbConnection: {},
            socketConnection: {},
            active: true
        }
    }
    
    pub fn start(&self) {
        self.hfpProxyService.start();
        self.wifiHotspot.start();
        self.bluetoothService.start();
        self.strand.dispatch([this, self = this.shared_from_this()]() {
            AW_LOG(info) << "Starting";
    
            let promise = f1x::aasdk::usb::IUSBHub::Promise::defer(self.strand);
            promise.then(std::bind(&self.onUSBDeviceConnected, this.shared_from_this(), std::placeholders::_1), std::bind(&self.onUSBError, this.shared_from_this(), std::placeholders::_1));
            self.usbHub.start(std::move(promise));
            self.startServerSocket();
        });
    }
    
    pub fn stop(&self) {
        self.strand.dispatch([this, self = this.shared_from_this()]() {
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
    
    pub fn startServerSocket(&self) {
        self.strand.dispatch([this, self = this.shared_from_this()]() {
            AW_LOG(info) << "Listening for WIFI clients on port 5000";
            let socket = std::make_shared<boost::asio::ip::tcp::socket>(ioService);
            self.acceptor.async_accept(
                    *socket,
                    std::bind(&self.onNewSocket, this, socket, std::placeholders::_1)
            );
        });
    }
    
    pub fn onNewSocket(&self, socket: std::shared_ptr<boost::asio::ip::tcp::socket>, &err: boost::system::error_code ) {
        self.strand.dispatch([this, self = this.shared_from_this(), socket, err]() {
            if (!err) {
                AW_LOG(info) << "WIFI Client connected";
                self.socketConnection = self.connectionFactory.create(std::move(socket));
                self.tryStartProxy();
            } else {
                AW_LOG(error) << "Socket connection error: " << err;
            }
        });
    }
    
    pub fn tryStartProxy(&self, ) {
        if (self.usbConnection != ptr::null() && self.socketConnection != ptr::null()) {    
            //TODO: start error handling
            self.usbConnection.start();
            self.socketConnection.start();
    
            self.startUSBReceive();
            self.startTCPReceive();
        }
    
    }
    
    pub fn onUSBReceive(&self, message: f1x::aasdk::messenger::Message::Pointer) {
        if (self.active) {
            let promise = f1x::aasdk::messenger::SendPromise::defer(self.strand);
            promise.then([]() {}, std::bind(&self.onError, this.shared_from_this(), std::placeholders::_1));
    
            if (message.getChannelId() == f1x::aasdk::messenger::ChannelId::CONTROL) {
                let messageId = f1x::aasdk::messenger::MessageId(message.getPayload());
                let payload = f1x::aasdk::common::DataConstBuffer(message.getPayload(), messageId.getSizeOf());
                if (messageId.getId() == f1x::aasdk::proto::ids::ControlMessage::SERVICE_DISCOVERY_RESPONSE) {
                    let response: f1x::aasdk::proto::messages::ServiceDiscoveryResponse;
                    response.ParseFromArray(payload.cdata, payload.size);
    
                    for channel in response.mutable_channels() {
                        if (channel.channel_id() == static_cast<uint32_t>(f1x::aasdk::messenger::ChannelId::BLUETOOTH)) {
                            f1x::aasdk::proto::data::BluetoothChannel *self.bluetoothChannel = channel.mutable_bluetooth_channel();
                            self.bluetoothChannel.set_adapter_address(self.bluetoothService.getAddress()); //TODO: set address
                            self.bluetoothChannel.clear_supported_pairing_methods();
                            self.bluetoothChannel.add_supported_pairing_methods(f1x::aasdk::proto::enums::BluetoothPairingMethod_Enum_HFP);
                            self.bluetoothChannel.add_supported_pairing_methods(f1x::aasdk::proto::enums::BluetoothPairingMethod_Enum_A2DP);
                        }
                    }
    
                    self.socketConnection.send(message, promise);
                    self.startUSBReceive();
                    return;
                }
            }
    
            //TODO: handle messages
            self.socketConnection.send(message, promise);
            self.startUSBReceive();
        }
    }
    
    pub fn onTCPReceive(&self, message: f1x::aasdk::messenger::Message::Pointer) {
        if (self.active) {
            let promise = f1x::aasdk::messenger::SendPromise::defer(self.strand);
            promise.then([]() {}, std::bind(&self.onError, this.shared_from_this(), std::placeholders::_1));
    
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
    
                    self.socketConnection.send(std::move(msg), std::move(promise));
                    self.startTCPReceive();
                    return;
                }
            }
    
            self.usbConnection.send(std::move(message), std::move(promise));
            self.startTCPReceive();
        }
    }
    
    pub fn startUSBReceive(&self) {
        let receivePromise = f1x::aasdk::messenger::ReceivePromise::defer(self.strand);
        receivePromise.then(std::bind(&self.onUSBReceive, this.shared_from_this(), std::placeholders::_1),
                                std::bind(&self.onError, this.shared_from_this(), std::placeholders::_1));
        self.usbConnection.receive(receivePromise);
    }
    
    pub fn startTCPReceive(&self) {
        let receivePromise = f1x::aasdk::messenger::ReceivePromise::defer(self.strand);
        receivePromise.then(std::bind(&self.onTCPReceive, this.shared_from_this(), std::placeholders::_1),
                                std::bind(&self.onError, this.shared_from_this(), std::placeholders::_1));
        self.socketConnection.receive(receivePromise);
    }
    
    pub fn onError(&self, &error: f1x::aasdk::error::Error) {
        self.cleanup();
        AW_LOG(error) << "Connection error: " << error.getNativeCode();
    }
    
    pub fn cleanup(&self) {
        self.active = false;
        if (self.usbConnection != ptr::null()) {
            self.usbConnection.stop();
            self.usbConnection = ptr::null();
        }
        if (self.socketConnection != ptr::null()) {
            self.socketConnection.stop();
            self.socketConnection = ptr::null();
        }
    }
    
    pub fn onUSBDeviceConnected(&self, deviceHandle: f1x::aasdk::usb::DeviceHandle) {
        let usbConnection = self.connectionFactory.create(deviceHandle);
        self.tryStartProxy();
    }
    
    pub fn onUSBError(&self, &error: f1x::aasdk::error::Error) {
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