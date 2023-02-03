
// #include "HFPProxyService.h"
// #include "HFPProxyProfile.h"
// #include <BluezQt/PendingCall>
// #include <aawireless/log/Log.h>
// #include <QtBluetooth/QBluetoothSocket>

pub fn HFPProxyService(btManager: std::shared_ptr<BluezQt::Manager>) -> btManager(std::move(btManager)) {
}

pub fn start() {
    hfpProxyProfile = std::make_shared<HFPProxyProfile>();
    BluezQt::PendingCall *call = btManager->registerProfile(hfpProxyProfile.get());
    connect(call, &BluezQt::PendingCall::finished, this, &onProfileReady);
    connect(hfpProxyProfile.get(), &HFPProxyProfile::onNewRfcommSocket, this,
    &newRfcommSocket);
}

pub fn stop() {
    btManager->unregisterProfile(hfpProxyProfile.get());
}

pub fn connectToDevice(address: QString) {
    QBluetoothSocket socket;
    socket.connectToService(
        QBluetoothAddress(address),
        QBluetoothUuid(QBluetoothUuid::ServiceClassUuid::Handsfree)
    );
}

pub fn newRfcommSocket(socket: QSharedPointer<QLocalSocket>) {
}

pub fn onProfileReady(BluezQt::PendingCall *call) {
    if (call->error()) {
        AW_LOG(error) << "Error registering profile" << call->errorText().toStdString();
    return;
}

AW_LOG(info) << "HFP profile registered";
}