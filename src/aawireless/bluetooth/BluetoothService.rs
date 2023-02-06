
use std;
use google;
use f1x;
use crate::aawireless::configuration::*;
use crate::aawireless::database::*;
// #include <QtBluetooth/QBluetoothServiceInfo>
// #include "BluetoothService.h"
// #include <aawireless/log/Log.h>
// #include <WifiInfoRequestMessage.pb.h>
// #include <QtCore/QDataStream>
// #include <iomanip>
// #include <WifiInfoResponseMessage.pb.h>
// #include <WifiSecurityResponseMessage.pb.h>
// #include <QtDBus/QDBusConnection>
// #include <QtDBus/QDBusInterface>
// #include <QtDBus/QDBusReply>


pub struct BluetoothService {
    localDevice: QBluetoothLocalDevice,
    serviceInfo: QBluetoothServiceInfo,
    server: QBluetoothServer,
    buffer: QByteArray,
    socket: *mut QBluetoothSocket,
    configuration: &Configuration::Configuration,
    database: &Database::DatabaseX,
    password: String,
}
impl BluetoothService {
    pub fn new(
        &configuration: Configuration::Configuration,
        &database: Database::DatabaseX,
        password: String
    ) -> Self {
        connect(&server, &QBluetoothServer::newConnection, self, &BluetoothService::onClientConnected);
    }

    pub fn start(&self) {
        AW_LOG(info) << "Start listening for bluetooth connections";
        self.localDevice.powerOn();
        self.localDevice.setHostMode(QBluetoothLocalDevice::HostDiscoverable);
    
        self.server.listen(self.localDevice.address());
        self.registerService(self.server.serverPort());
    
        if (!self.database.getLastBluetoothDevice().empty()) {
            self.connectDevice(self.database.getLastBluetoothDevice());
        }
    }

    fn connectDevice(&self, address: String) {
        AW_LOG(info) << "Connecting to " << address;
        std::replace(address.begin(), address.end(), ':', '_');
        let iface = QDBusInterface("org.bluez",String("/org/bluez/hci0/dev_").append(address).c_str(), "org.bluez.Device1", QDBusConnection::systemBus());
        if (iface.isValid()) {
            QDBusReply<void> reply = iface.call("Connect");
            if (!reply.isValid()) {
                AW_LOG(error) << reply.error().message().toStdString();
            }
        } else {
            AW_LOG(error) << "Invalid interface" << iface.lastError().message().toStdString();
        }
    }
    
    pub fn stop(&self) {
        serviceInfo.unregisterService();
    }
    
    pub fn onClientConnected(&self) {
        if (self.socket != std::ptr::null()) {
            self.socket.deleteLater();
        }
    
        self.socket = server.nextPendingConnection();
    
        self.database.setLastBluetoothDevice(self.socket.peerAddress().toString().toStdString());
        self.database.save();
    
        if (self.socket != std::ptr::null()) {
            AW_LOG(info) << "[AndroidBluetoothServer] rfcomm client connected, peer name: " << socket.peerName().toStdString();
    
            self.connect(socket, &QBluetoothSocket::readyRead, this, &BluetoothService::readSocket);
            //  connect(socket, &QBluetoothSocket::disconnected, this, QOverload<>::of(&ChatServer::clientDisconnected));
    
            let request: f1x::aasdk::proto::messages::WifiInfoRequest;
            request.set_ip_address(self.configuration.wifiIpAddress);
            request.set_port(self.configuration.wifiPort);
    
            self.sendMessage(request, 1);
        } else {
            AW_LOG(error) << "received null socket during client connection.";
        }
    }
    
    fn readSocket(&self) {
        self.buffer += self.socket.readAll();
    
        AW_LOG(info) << "Received message";
    
        if (self.buffer.length() < 4) {
            AW_LOG(debug) << "Not enough data, waiting for more";
            return;
        }
    
        let stream = QDataStream(self.buffer);
        let mut length: u16;
        stream >> length;
    
        if (self.buffer.length() < length + 4) {
            AW_LOG(info) << "Not enough data, waiting for more: " << self.buffer.length();
            return;
        }
    
        let mut messageId: u32;
        stream >> messageId;
    
        //OPENAUTO_LOG(info) << "[AndroidBluetoothServer] " << length << " " << messageId;
    
        match messageId {
            1=>
                self.handleWifiInfoRequest(buffer, length),
            2=>
                self.handleWifiSecurityRequest(buffer, length),
            7=>
                self.handleWifiInfoRequestResponse(buffer, length),
            _=> {
                let ss: std::stringstream;
                ss << std::hex << std::setfill('0');
                for val in self.buffer {
                    ss << std::setw(2) << static_cast<unsigned>(val);
                }
                AW_LOG(info) << "Unknown message: " << messageId;
                AW_LOG(info) << ss.str();
            }
        }
    
        self.buffer = self.buffer.mid(length + 4);
    }
    
    fn handleWifiInfoRequest(&self, &buffer: QByteArray, length: u16) {
        let msg: f1x::aasdk::proto::messages::WifiInfoRequest;
        msg.ParseFromArray(buffer.data() + 4, length);
        AW_LOG(info) << "WifiInfoRequest: " << msg.DebugString();
    
        let response: f1x::aasdk::proto::messages::WifiInfoResponse;
        response.set_ip_address(self.configuration.wifiIpAddress);
        response.set_port(self.configuration.wifiPort);
        response.set_status(f1x::aasdk::proto::messages::WifiInfoResponse_Status_STATUS_SUCCESS);
    
        self.sendMessage(response, 7);
    }
    
    fn handleWifiSecurityRequest(&self, &buffer: QByteArray, length: u16) {
        let response: f1x::aasdk::proto::messages::WifiSecurityReponse;
    
        response.set_ssid(self.configuration.wifiSSID);
        response.set_bssid(self.configuration.wifiBSSID);
        response.set_key(self.configuration.wifiPassphrase);
        response.set_security_mode(
        f1x::aasdk::proto::messages::WifiSecurityReponse_SecurityMode_WPA2_PERSONAL); //TODO: make configurable?
        response.set_access_point_type(f1x::aasdk::proto::messages::WifiSecurityReponse_AccessPointType_STATIC);
    
        self.sendMessage(response, 3);
    }
    
    fn sendMessage(&self, &message: google::protobuf::Message, sm_type: u16) {
        let byteSize = message.ByteSize();
        let out = QByteArray(byteSize + 4, 0);
        let ds = QDataStream(&out, QIODevice::ReadWrite);
        ds << byteSize as u16;
        ds << sm_type;
        message.SerializeToArray(out.data() + 4, byteSize);
    
        let ss: std::stringstream;
        ss << std::hex << std::setfill('0');
        for val in out {
            ss << std::setw(2) << static_cast<unsigned>(val);
        }
        AW_LOG(info) << "Writing message: " << ss.str();
    
        let written = self.socket.write(out);
        if (written > -1) {
            AW_LOG(info) << "Bytes written: " << written;
        } else {
            AW_LOG(info) << "Could not write data";
        }
    }
    
    fn handleWifiInfoRequestResponse(&self, &buffer: QByteArray, length: u16) {
        let msg: f1x::aasdk::proto::messages::WifiInfoResponse;
        msg.ParseFromArray(buffer.data() + 4, length);
        AW_LOG(info) << "WifiInfoResponse: " << msg.DebugString();
    }
    
    fn registerService(&self, port: quint16) {
        let serviceUuid = QBluetoothUuid(QLatin1String("4de17a00-52cb-11e6-bdf4-0800200c9a66"));
    
        let classId: QBluetoothServiceInfo::Sequence;
        classId << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::SerialPort));
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::BluetoothProfileDescriptorList, classId);
        classId.prepend(QVariant::fromValue(serviceUuid));
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceClassIds, classId);
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceName, "AAWireless Bluetooth Service");
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceDescription, "AndroidAuto WiFi projection automatic setup");
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceProvider, "AAWireless");
        self.serviceInfo.setServiceUuid(serviceUuid);
    
        let publicBrowse: QBluetoothServiceInfo::Sequence;
        publicBrowse << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::PublicBrowseGroup));
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::BrowseGroupList, publicBrowse);
    
        let protocolDescriptorList: QBluetoothServiceInfo::Sequence ;
        let protocol: QBluetoothServiceInfo::Sequence;
        protocol << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::L2cap));
        protocolDescriptorList.append(QVariant::fromValue(protocol));
        protocol.clear();
        protocol << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::Rfcomm)) << QVariant::fromValue(port);
        protocolDescriptorList.append(QVariant::fromValue(protocol));
        self.serviceInfo.setAttribute(QBluetoothServiceInfo::ProtocolDescriptorList, protocolDescriptorList);
    
        self.serviceInfo.registerService(self.localDevice.address());
    }
    
    pub fn getAddress(&self) -> String {
        return self.localDevice.address().toString().toStdString();
    }
}
