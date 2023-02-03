pub fn BluetoothService(
    &configuration: aawireless::configuration::Configuration,
    &database: aawireless::database::Database,
    password: std::string
) -> server(QBluetoothServiceInfo::RfcommProtocol), configuration(configuration), database(database), password(password) {
    connect(&server, &QBluetoothServer::newConnection, this, &BluetoothService::onClientConnected);
}

pub fn start() {
    AW_LOG(info) << "Start listening for bluetooth connections";
    localDevice.powerOn();
    localDevice.setHostMode(QBluetoothLocalDevice::HostDiscoverable);

    server.listen(localDevice.address());
    registerService(server.serverPort());

    if (!database.getLastBluetoothDevice().empty()) {
        connectDevice(database.getLastBluetoothDevice());
    }
}

pub fn connectDevice(address: std::string) {
    AW_LOG(info) << "Connecting to " << address;
    std::replace(address.begin(), address.end(), ':', '_');
    QDBusInterface iface("org.bluez",std::string("/org/bluez/hci0/dev_").append(address).c_str(),"org.bluez.Device1",QDBusConnection::systemBus());
    if (iface.isValid()) {
        QDBusReply<void> reply = iface.call("Connect");
        if (!reply.isValid()) {
            AW_LOG(error) << reply.error().message().toStdString();
        }
    } else {
        AW_LOG(error) << "Invalid interface" << iface.lastError().message().toStdString();
    }
}

pub fn stop() {
    serviceInfo.unregisterService();
}

pub fn onClientConnected() {
    if (socket != nullptr) {
        socket->deleteLater();
    }

    socket = server.nextPendingConnection();

    database.setLastBluetoothDevice(socket->peerAddress().toString().toStdString());
    database.save();

    if (socket != nullptr) {
        AW_LOG(info) << "[AndroidBluetoothServer] rfcomm client connected, peer name: "
        << socket->peerName().toStdString();

        connect(socket, &QBluetoothSocket::readyRead, this, &BluetoothService::readSocket);
        //                    connect(socket, &QBluetoothSocket::disconnected, this,
        //                            QOverload<>::of(&ChatServer::clientDisconnected));

        let request: f1x::aasdk::proto::messages::WifiInfoRequest;
        request.set_ip_address(configuration.wifiIpAddress);
        request.set_port(configuration.wifiPort);

        sendMessage(request, 1);
    } else {
        AW_LOG(error) << "received null socket during client connection.";
    }
}

pub fn readSocket() {
    buffer += socket->readAll();

    AW_LOG(info) << "Received message";

    if (buffer.length() < 4) {
        AW_LOG(debug) << "Not enough data, waiting for more";
        return;
    }

    QDataStream stream(buffer);
    let length: u16;
    stream >> length;

    if (buffer.length() < length + 4) {
        AW_LOG(info) << "Not enough data, waiting for more: " << buffer.length();
        return;
    }

    let messageId: u32;
    stream >> messageId;

    //OPENAUTO_LOG(info) << "[AndroidBluetoothServer] " << length << " " << messageId;

    match messageId {
        1=>
            handleWifiInfoRequest(buffer, length),
        2=>
            handleWifiSecurityRequest(buffer, length),
        7=>
            handleWifiInfoRequestResponse(buffer, length),
        _=> {
            let ss: std::stringstream;
            ss << std::hex << std::setfill('0');
            for val in buffer {
                ss << std::setw(2) << static_cast<unsigned>(val);
            }
            AW_LOG(info) << "Unknown message: " << messageId;
            AW_LOG(info) << ss.str();
        }
    }

    buffer = buffer.mid(length + 4);
}

pub fn handleWifiInfoRequest(&buffer: QByteArray, length: uint16_t) {
    let msg: f1x::aasdk::proto::messages::WifiInfoRequest;
    msg.ParseFromArray(buffer.data() + 4, length);
    AW_LOG(info) << "WifiInfoRequest: " << msg.DebugString();

    let response: f1x::aasdk::proto::messages::WifiInfoResponse;
    response.set_ip_address(configuration.wifiIpAddress);
    response.set_port(configuration.wifiPort);
    response.set_status(f1x::aasdk::proto::messages::WifiInfoResponse_Status_STATUS_SUCCESS);

    sendMessage(response, 7);
}

pub fn handleWifiSecurityRequest(&buffer: QByteArray, length: uint16_t) {
    let response: f1x::aasdk::proto::messages::WifiSecurityReponse;

    response.set_ssid(configuration.wifiSSID);
    response.set_bssid(configuration.wifiBSSID);
    response.set_key(configuration.wifiPassphrase);
    response.set_security_mode(
    f1x::aasdk::proto::messages::WifiSecurityReponse_SecurityMode_WPA2_PERSONAL); //TODO: make configurable?
    response.set_access_point_type(f1x::aasdk::proto::messages::WifiSecurityReponse_AccessPointType_STATIC);

    sendMessage(response, 3);
}

pub fn sendMessage(&message: google::protobuf::Message, type: u16) {
    let byteSize = message.ByteSize();
    QByteArray out(byteSize + 4, 0);
    QDataStream ds(&out, QIODevice::ReadWrite);
    ds << byteSize as u16;
    ds << type;
    message.SerializeToArray(out.data() + 4, byteSize);

    let ss: std::stringstream;
    ss << std::hex << std::setfill('0');
    for val in out {
        ss << std::setw(2) << static_cast<unsigned>(val);
    }
    AW_LOG(info) << "Writing message: " << ss.str();

    let written: auto = socket->write(out);
    if (written > -1) {
        AW_LOG(info) << "Bytes written: " << written;
    } else {
        AW_LOG(info) << "Could not write data";
    }
}

pub fn handleWifiInfoRequestResponse(&buffer: QByteArray, length: u16) {
    let msg: f1x::aasdk::proto::messages::WifiInfoResponse;
    msg.ParseFromArray(buffer.data() + 4, length);
    AW_LOG(info) << "WifiInfoResponse: " << msg.DebugString();
}

pub fn registerService(port: quint16) {
    const QBluetoothUuid serviceUuid(QLatin1String("4de17a00-52cb-11e6-bdf4-0800200c9a66"));

    let classId: QBluetoothServiceInfo::Sequence;
    classId << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::SerialPort));
    serviceInfo.setAttribute(QBluetoothServiceInfo::BluetoothProfileDescriptorList, classId);
    classId.prepend(QVariant::fromValue(serviceUuid));
    serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceClassIds, classId);
    serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceName, "AAWireless Bluetooth Service");
    serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceDescription,
    "AndroidAuto WiFi projection automatic setup");
    serviceInfo.setAttribute(QBluetoothServiceInfo::ServiceProvider, "AAWireless");
    serviceInfo.setServiceUuid(serviceUuid);

    let publicBrowse: QBluetoothServiceInfo::Sequence;
    publicBrowse << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::PublicBrowseGroup));
    serviceInfo.setAttribute(QBluetoothServiceInfo::BrowseGroupList, publicBrowse);

    let protocolDescriptorList: QBluetoothServiceInfo::Sequence ;
    let protocol: QBluetoothServiceInfo::Sequence;
    protocol << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::L2cap));
    protocolDescriptorList.append(QVariant::fromValue(protocol));
    protocol.clear();
    protocol << QVariant::fromValue(QBluetoothUuid(QBluetoothUuid::Rfcomm))
    << QVariant::fromValue(port);
    protocolDescriptorList.append(QVariant::fromValue(protocol));
    serviceInfo.setAttribute(QBluetoothServiceInfo::ProtocolDescriptorList, protocolDescriptorList);

    serviceInfo.registerService(localDevice.address());
}

pub fn getAddress() -> std::string {
    return localDevice.address().toString().toStdString();
}