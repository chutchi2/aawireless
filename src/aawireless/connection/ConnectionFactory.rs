use f1x;
use std;
use boost;

// #include "ConnectionFactory.h"
// #include <f1x/aasdk/Transport/SSLWrapper.hpp>
// #include <f1x/aasdk/Messenger/Cryptor.hpp>
// #include <f1x/aasdk/Messenger/Messenger.hpp>
// #include <f1x/aasdk/Messenger/MessageInStream.hpp>
// #include <f1x/aasdk/Messenger/MessageOutStream.hpp>
// #include <f1x/aasdk/Transport/TCPTransport.hpp>
// #include <f1x/aasdk/TCP/TCPEndpoint.hpp>
// #include <f1x/aasdk/USB/IAOAPDevice.hpp>
// #include <f1x/aasdk/Transport/USBTransport.hpp>
// #include <f1x/aasdk/USB/AOAPDevice.hpp>

pub struct ConnectionFactory {
    ioService: &boost::asio::io_service,
    tcpWrapper: &f1x::aasdk::tcp::TCPWrapper,
    usbWrapper: &f1x::aasdk::usb::USBWrapper,
}

impl ConnectionFactory {
    pub fn new (
        &ioService: boost::asio::io_service ,
        &tcpWrapper: f1x::aasdk::tcp::TCPWrapper,
        &usbWrapper: f1x::aasdk::usb::USBWrapper) -> Self {
        Self {
            ioService: ioService,
            tcpWrapper: tcpWrapper,
            usbWrapper: usbWrapper
        }
    }
    
    pub fn create(&self, deviceHandle: f1x::aasdk::usb::DeviceHandle) -> std::shared_ptr<Connection> {
        let aoapDevice = (f1x::aasdk::usb::AOAPDevice::create(self.usbWrapper, self.ioService, deviceHandle));
        let transport = (std::make_shared<f1x::aasdk::transport::USBTransport>(ioService, std::move(aoapDevice)));
        return create(std::move(transport));
    }
    
    pub fn create(&self, socket: std::shared_ptr<boost::asio::ip::tcp::socket>) -> std::shared_ptr<Connection> {
        let endpoint = (std::make_shared<f1x::aasdk::tcp::TCPEndpoint>(tcpWrapper, std::move(socket)));
        let transport = (std::make_shared<f1x::aasdk::transport::TCPTransport>(ioService, std::move(endpoint)));
        return self.create(std::move(transport));
    }
    
    pub fn create(&self, transport: std::shared_ptr<f1x::aasdk::transport::ITransport>) -> std::shared_ptr<Connection> {
        let sslWrapper = (std::make_shared<f1x::aasdk::transport::SSLWrapper>());
        let cryptor = (std::make_shared<f1x::aasdk::messenger::Cryptor>(std::move(sslWrapper)));
    
        let inStream = (std::make_shared<f1x::aasdk::messenger::MessageInStream>(ioService, transport, cryptor));
        let outStream = (std::make_shared<f1x::aasdk::messenger::MessageOutStream>(ioService, transport, cryptor));
        return std::make_shared<Connection>(ioService, std::move(cryptor), std::move(transport), std::move(inStream), std::move(outStream));
    }
}
