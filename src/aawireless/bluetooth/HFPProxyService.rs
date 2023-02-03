
// #include "HFPProxyService.h"
// #include "HFPProxyProfile.h"
// #include <BluezQt/PendingCall>
// #include <aawireless/log/Log.h>
// #include <QtBluetooth/QBluetoothSocket>

struct HFPProxyService {
    btManager: std::shared_ptr<BluezQt::Manager>,
    hfpProxyProfile: std::shared_ptr<HFPProxyProfile>,
}
impl HFPProxyService {
    pub fn new(btManager: std::shared_ptr<BluezQt::Manager>) -> Self {
        btManager(std::move(btManager));
    }
    pub fn start(&self) {
        hfpProxyProfile = std::make_shared<HFPProxyProfile>();
        BluezQt::PendingCall *call = btManager.registerProfile(hfpProxyProfile.get());
        connect(call, &BluezQt::PendingCall::finished, this, &onProfileReady);
        connect(hfpProxyProfile.get(), &HFPProxyProfile::onNewRfcommSocket, this,
        &newRfcommSocket);
    }
    
    pub fn stop() {
        btManager.unregisterProfile(hfpProxyProfile.get());
    }
    
    pub fn connectToDevice(&self, address: QString) {
        let socket: QBluetoothSocket;
        socket.connectToService(
            QBluetoothAddress(address),
            QBluetoothUuid(QBluetoothUuid::ServiceClassUuid::Handsfree)
        );
    }
    
    pub fn newRfcommSocket(&self, socket: QSharedPointer<QLocalSocket>) {
    }
    
    pub fn onProfileReady(&self, call: *mut BluezQt::PendingCall) {
        if (call.error()) {
            AW_LOG(error) << "Error registering profile" << call.errorText().toStdString();
            return;
        }
    
        AW_LOG(info) << "HFP profile registered";
    }
}

