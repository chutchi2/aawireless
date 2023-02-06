
// #include "HFPProxyService.h"
// #include "HFPProxyProfile.h"
// #include <BluezQt/PendingCall>
// #include <aawireless/log/Log.h>
// #include <QtBluetooth/QBluetoothSocket>

pub struct HFPProxyService {
    btManager: std::shared_ptr<BluezQt::Manager>,
    hfpProxyProfile: std::shared_ptr<HFPProxyProfile>,
}

impl HFPProxyService {
    pub fn new(btManager: std::shared_ptr<BluezQt::Manager>) -> Self {
        Self {
            btManager: std::move(btManager),
            hfpProxyProfile: std::ptr::null(),
        }
    }
    pub fn start(&self) {
        self.hfpProxyProfile = std::make_shared<HFPProxyProfile>();
        let call: *mut BluezQt::PendingCall = self.btManager.registerProfile(hfpProxyProfile.get());
        connect(call, &BluezQt::PendingCall::finished, this, &self.onProfileReady);
        connect(hfpProxyProfile.get(), &HFPProxyProfile::onNewRfcommSocket, this, &self.newRfcommSocket);
    }
    
    pub fn stop() {
        self.btManager.unregisterProfile(hfpProxyProfile.get());
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

