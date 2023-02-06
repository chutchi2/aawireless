
// #include "BluetoothSocket.h"
// #include <errno.h>
// #include <unistd.h>
// #include <string.h>
// #include <QtCore/QSocketNotifier>
// #include <qplatformdefs.h>
// #include <bluetooth/bluetooth.h>
// #include <bluetooth/rfcomm.h>
// #include <aawireless/log/Log.h>
use std::vec;

enum SocketState {
    UnconnectedState = QAbstractSocket::UnconnectedState,
    ServiceLookupState = QAbstractSocket::HostLookupState,
    ConnectingState = QAbstractSocket::ConnectingState,
    ConnectedState = QAbstractSocket::ConnectedState,
    BoundState = QAbstractSocket::BoundState,
    ClosingState = QAbstractSocket::ClosingState,
    ListeningState = QAbstractSocket::ListeningState
}

enum SocketError {
    NoSocketError = -2,
    UnknownSocketError = QAbstractSocket::UnknownSocketError, //-1
    RemoteHostClosedError = QAbstractSocket::RemoteHostClosedError, //1
    HostNotFoundError = QAbstractSocket::HostNotFoundError, //2
    ServiceNotFoundError = QAbstractSocket::SocketAddressNotAvailableError, //9
    NetworkError = QAbstractSocket::NetworkError, //7
    UnsupportedProtocolError = 8,
    OperationError = QAbstractSocket::OperationError //19
    //New enums (independent of QAbstractSocket) should be added from 100 onwards
}

enum Security {
    NoSecurity = 0x00,
    Authorization = 0x01,
    Authentication = 0x02,
    Encryption = 0x04,
    Secure = 0x08
}

pub struct BluetoothSocket {
    buffer: QPrivateLinearBuffer,
    txBuffer: QPrivateLinearBuffer,
    secFlags: Security,
    socket: i32,
    socketError: SocketError,
    state: SocketState,
    readNotifier: std::unique_ptr<QSocketNotifier>,
    connectWriteNotifier: std::unique_ptr<QSocketNotifier>,
}

impl BluetoothSocket {
    pub fn new() -> Self {
        let readNotifier = std::make_unique<QSocketNotifier>(socket, QSocketNotifier::Read);
        connect(readNotifier.get(), SIGNAL(activated(int)), this, SLOT(readNotify()));
        connectWriteNotifier = std::make_unique<QSocketNotifier>(socket, QSocketNotifier::Write);
        connect(connectWriteNotifier.get(), SIGNAL(activated(int)), this, SLOT(writeNotify));
        connectWriteNotifier.setEnabled(false);
        readNotifier.setEnabled(false);
        return Self {
            secFlags: Security::Authorization,
            socketError: Security::NoSocketError,
            state: Security::UnconnectedState,
        };
    }
    
    
    //TODO: do something with security?
    //pub fn connectToAddress() {
        // apply preferred security level
        // ignore QBluetooth::Authentication -> not used anymore by kernel
    //            struct bt_security security;
    //            memset(&security, 0, sizeof(security));
    //
    //            if (secFlags & Security::Authorization){
    //                security.level = BT_SECURITY_LOW;}
    //            if (secFlags & Security::Encryption)
    //                {security.level = BT_SECURITY_MEDIUM;}
    //            if (secFlags & Security::Secure)
    //                {security.level = BT_SECURITY_HIGH;}
    //
    //            if (setsockopt(socket, SOL_BLUETOOTH, BT_SECURITY,
    //                           &security, sizeof(security)) != 0) {
    //                AW_LOG(error) << "Cannot set connection security level, closing socket for safety"
    //                              << qt_error_string(errno).toStdString();
    //                socketError = UnknownSocketError;
    //                return;
    //            }
    //  }
    
    pub fn writeNotify(&self) {
        if (self.state == SocketState::ConnectingState) {
            let errorno;
            let len;
            len = sizeof(errorno);
            ::getsockopt(self.socket, SOL_SOCKET, SO_ERROR, &errorno, &len as *mut socklen_t);
            if (errorno) {
                AW_LOG(error) << "Could not complete connection to socket " << qt_error_string(errorno).toStdString();
                self.setSocketError(SocketError::UnknownSocketError);
                return;
            }
    
            self.setSocketState(SocketState::ConnectedState);
    
            self.connectWriteNotifier.setEnabled(false);
        } else {
            if (self.txBuffer.size() == 0) {
                self.connectWriteNotifier.setEnabled(false);
                return;
            }
            let buf: Vec<i32> = vec![];
    
            let size: i32 = self.txBuffer.read(buf, 1024);
            //TODO: int writtenBytes = qt_safe_write(socket, buf, size);
            let mut writtenBytes: i32 = 0;
            if (writtenBytes < 0) {
                match errno {
                    EAGAIN => {
                            writtenBytes = 0;
                            self.txBuffer.ungetBlock(buf, size);
                        }
                    _ =>
                        // every other case returns error
                        self.setSocketError(SocketError::NetworkError),
                }
            } else {
                if (writtenBytes < size) {
                    // add remainder back to buffer
                    let remainder: *mut char = buf + writtenBytes;
                    self.txBuffer.ungetBlock(remainder, size - writtenBytes);
                }
            }
    
            if (self.txBuffer.size()) {
                self.connectWriteNotifier.setEnabled(true);
            } else if (self.state == SocketState::ClosingState) {
                self.connectWriteNotifier.setEnabled(false);
                self.close();
            }
        }
    }
    
    pub fn readNotify(&self) {
        let writePointer: *mut char = self.buffer.reserve(QPRIVATELINEARBUFFER_BUFFERSIZE);
        let readFromDevice: i32 = ::read(socket, writePointer, QPRIVATELINEARBUFFER_BUFFERSIZE);
        self.buffer.chop(QPRIVATELINEARBUFFER_BUFFERSIZE - (if readFromDevice < 0 { 0 } else { readFromDevice }));
        if (readFromDevice <= 0) {
            let errsv: i32 = errno;
            self.readNotifier.setEnabled(false);
            self.connectWriteNotifier.setEnabled(false);
            AW_LOG(error) << "Could not read from device " << qt_error_string(errsv).toStdString();
            if (errsv == EHOSTDOWN){
                self.setSocketError(SocketError::HostNotFoundError);
            }else if (errsv == ECONNRESET){
                self.setSocketError(SocketError::RemoteHostClosedError);
            }else{
                self.setSocketError(SocketError::UnknownSocketError);
            }
        } else {
            emit readyRead();
        }
    }
    
    pub fn abort(&self) {
        self.readNotifier = std::ptr::null();
        self.connectWriteNotifier = std::ptr::null();
    
        // We don't transition through Closing for abort, so
        // we don't call disconnectFromService or
        // Qclose
        QT_CLOSE(self.socket);
        self.socket = -1;
    
        self.setSocketState(SocketState::UnconnectedState);
        emit self.disconnected();
    }
    
    pub fn writeData(&self, data: *mut char, maxSize: qint64) -> qint64 {
        if (self.state != SocketState::ConnectedState) {
            AW_LOG(error) << "Cannot write while not connected";
            self.setSocketError(SocketError::OperationError);
            return -1;
        }
    
        if (!self.connectWriteNotifier){
            return -1;
        }
    
        if (self.txBuffer.size() == 0) {
            self.connectWriteNotifier.setEnabled(true);
            QMetaObject::invokeMethod(this, "writeNotify", Qt::QueuedConnection);
        }
    
        let txbuf: *mut char = self.txBuffer.reserve(maxSize);
        std::memcpy(txbuf, data, maxSize);
    
        return maxSize;
    }
    
    pub fn readData(&self, data: *mut char, maxSize: qint64) -> qint64 {
        if (self.state != SocketState::ConnectedState) {
            AW_LOG(error) << "Cannot read while not connected";
            self.setSocketError(SocketError::OperationError);
            return -1;
        }
    
        if (!self.buffer.isEmpty()) {
            let i: i32 = self.buffer.read(data, maxSize);
            return i;
        }
    
        return 0;
    }
    
    pub fn close(&self) {
        if (self.txBuffer.size() > 0){
            self.connectWriteNotifier.setEnabled(true);
        }
        else{
            self.abort();
        }
    }
    
    pub fn setSocketError(&self, _error: SocketError) {
        self.socketError = _error;
        emit self.error(socketError);
    }
    
    pub fn setSocketState(&self, _state: SocketState) {
        let old = self.state;
        if (_state == old){
            return;
        }
        self.state = _state;
    
        emit self.stateChanged(state);
        if (state == ConnectedState) {
            emit connected();
        } else if ((old == ConnectedState || old == ClosingState) && state == UnconnectedState) {
            emit disconnected();
        }
        if (state == ListeningState) {
            // TODO: look at this, is this really correct?
            // if we're a listening socket we can't handle connects?
            if (readNotifier) {
                readNotifier.setEnabled(false);
            }
        }
    }
    
    // inline convertAddress
    pub fn convertAddress(&self, address: String, out: bdaddr_t) {
        let src_addr: *mut char = address.c_str();
    
        /* don't use ba2str to apub fn -lbluetooth */
        let mut i = 5;
        while (i >= 0){
            src_addr += 3;
            i -= 1;
            out.b[i] = std::strtol(src_addr, std::ptr::null(), 16);
        }
        // let mut i = 5;
        // while (i >= 0){
        //     i -= 1;
        //     src_addr += 3;
        //     out.b[i] = strtol(src_addr, NULL, 16);
        // }
    }
    
    pub fn connectRfcomm(&self, address: String, channel: u8) {
        impl addr for sockaddr_rc{};
        std::memset(&addr, 0, std::sizeof(addr));
        addr.rc_family = AF_BLUETOOTH;
        addr.rc_channel = channel;
        self.convertAddress(address, addr.rc_bdaddr);
    
        self.socket = ::socket(AF_BLUETOOTH, SOCK_STREAM, BTPROTO_RFCOMM);
    
        self.connectWriteNotifier.setEnabled(true);
        self.readNotifier.setEnabled(true);
    
        let result: u32 = ::connect(self.socket, &addr as sockaddr, std::sizeof(addr));
    
        if (result >= 0 || (result == -1 && errno == EINPROGRESS)) {
            self.setSocketState(SocketState::ConnectingState);
        } else {
            AW_LOG(error) << "Could not open socket " << qt_error_string(errno).toStdString();
            self.setSocketError(SocketError::UnknownSocketError);
        }
    }
    
    pub fn connectSCO(&self, address: String) {
        impl addr for sockaddr_sco{};
        std::memset(&addr, 0, std::sizeof(addr));
        addr.sco_family = AF_BLUETOOTH;
        self.convertAddress(address, addr.sco_bdaddr);
    
        socket = ::socket(AF_BLUETOOTH, SOCK_SEQPACKET, BTPROTO_SCO);
    
        connectWriteNotifier.setEnabled(true);
        readNotifier.setEnabled(true);
    
        let result: i32 = ::connect(socket, &addr as sockaddr, sizeof(addr));
    
        if (result >= 0 || (result == -1 && errno == EINPROGRESS)) {
            self.setSocketState(SocketState::ConnectingState);
        } else {
            AW_LOG(error) << "Could not open socket " << qt_error_string(errno).toStdString();
            self.setSocketError(SocketError::UnknownSocketError);
        }
    }
}