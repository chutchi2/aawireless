// #include <QCoreApplication>
// #include <QtDBus/QDBusConnection>
// #include <thread>
// #include <libusb.h>
// #include "aawireless/bluetooth/BluetoothService.h"
// #include <boost/asio/io_service.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include <aawireless/log/Log.h>
// #include <f1x/aasdk/TCP/TCPWrapper.hpp>
// #include <f1x/aasdk/USB/USBWrapper.hpp>
// #include <f1x/aasdk/USB/USBHub.hpp>
// #include <f1x/aasdk/USB/AccessoryModeQueryFactory.hpp>
// #include <f1x/aasdk/USB/AccessoryModeQueryChainFactory.hpp>
// #include <f1x/aasdk/USB/ConnectedAccessoriesEnumerator.hpp>
// #include <aawireless/App.h>
// #include <aawireless/connection/ConnectionFactory.h>
// #include <aawireless/configuration/Configuration.h>
// #include <aawireless/database/Database.h>
// #include <aawireless/wifi/WifiHotspot.h>
// #include "boost/random/random_device.hpp"
// #include "boost/random/uniform_int_distribution.hpp"
// #include <BluezQt/Manager>
// #include <BluezQt/InitManagerJob>
// #include <BluezQt/PendingCall>
// #include <aawireless/bluetooth/HFPProxyProfile.h>
// #include <aawireless/bluetooth/HFPProxyService.h>
mod aawireless;
use crate::aawireless::bluetooth::*;
use crate::aawireless::connection::*;
use crate::aawireless::configuration::*;
use crate::aawireless::wifi::*;
use crate::aawireless::database::*;
// use crate::aawireless::App;
use std::{vec, ptr::null};
use boost;
use f1x;

// using ThreadPool = std::Vec<std::thread>;

pub fn startUSBWorkers(&ioService: boost::asio::io_service, usbContext: *mut libusb_context, threadPool: &Vec<std::thread::Thread>) {
    fn usbWorker(ioService: &boost::asio::io_service, usbContext: *mut libusb_context) {
        let libusbEventTimeout = timeval{180, 0};

        while (!ioService.stopped()) {
            libusb_handle_events_timeout_completed(usbContext, &libusbEventTimeout, null);
        }
    }

    threadPool.emplace_back(std::thread::Thread(usbWorker));
}

pub fn startIOServiceWorkers(&ioService: boost::asio::io_service, threadPool: &Vec<std::thread::Thread>) {
    fn ioServiceWorker (&ioService: boost::asio::io_service) {
        ioService.run();
    };

    threadPool.emplace_back(std::thread::Thread(ioServiceWorker));
}

//TODO: refactor to other location
pub fn generatePassword() -> std::string::String {
    let chars = std::string::String(
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "1234567890",
            "!@#$%^&*()");
    //"`~-_=+[{]{\\|;:'\",<.>/? ");
    /*<< We use __random_device as a source of entropy, since we want
        passwords that are not predictable.
    >>*/
    let rng: boost::random::random_device;
    /*<< Finally we select 8 random characters from the
        string and print them to cout.
    >>*/
    let index_dist = boost::random::uniform_int_distribution(0, chars.size() - 1);
    let ss: std::stringstream;
    for i in 0..12 {
        ss << chars[index_dist(rng)];
    }
    return ss.str();
}

pub fn main(argc: int, argv: *mut Vec<char>) -> i32 {
    if (!QDBusConnection::systemBus().isConnected()) {
        AW_LOG(error) << "Cannot connect to the D-Bus session bus.";
        return 1;
    }

    let usbContext: *mut libusb_context;
    if (libusb_init(&usbContext) != 0) {
        AW_LOG(error) << "[OpenAuto] libusb init failed.";
        return 1;
    }

    let mut ioService: boost::asio::io_service;
    let work = boost::asio::io_service::work(ioService);
    let mut threadPool: Vec<std::thread::Thread>;
    startUSBWorkers(ioService, usbContext, threadPool);
    startIOServiceWorkers(ioService, threadPool);

    let qApplication = QCoreApplication(argc, argv);

    let password: std::string::String = generatePassword();
    AW_LOG(info) << "Wifi password " << password;

    let configuration = Configuration::Configuration("config.ini");
    let database = Database::DatabaseX("/var/lib/aawireless/db.ini");
    let tcpWrapper = f1x::aasdk::tcp::TCPWrapper;
    let usbWrapper = f1x::aasdk::usb::USBWrapper(usbContext);
    let queryFactory = f1x::aasdk::usb::AccessoryModeQueryFactory(usbWrapper, ioService);
    let queryChainFactory = f1x::aasdk::usb::AccessoryModeQueryChainFactory(usbWrapper, ioService, queryFactory);
    let acceptor = boost::asio::ip::tcp::acceptor(ioService, boost::asio::ip::tcp::endpoint(boost::asio::ip::tcp::v4(), 5000));
    let bluetoothService = BluetoothService::BluetoothService(configuration, database, password);
    let wifiHotspot = WifiHotspot::WifiHotspot(ioService, configuration, password);
    let usbHub = std::make_shared<f1x::aasdk::usb::USBHub>(usbWrapper, ioService, queryChainFactory);
    let connectionFactory = ConnectionFactory::ConnectionFactory(ioService, tcpWrapper, usbWrapper);

    let btManager = std::make_shared<BluezQt::Manager>();
    let initJob = btManager.init(); // TODO: refactor to InitManagerJob.start()
    initJob.exec();
    if (initJob.error()) {
        AW_LOG(error) << "Error running bt init job" << initJob.errorText().toStdString();
        return 1;
    }
    let hfpProxyService = HFPProxyService::HFPProxyService(btManager);

    let app = std::make_shared::<aawireless::App::App>(
        ioService,
        usbHub,
        acceptor,
        wifiHotspot,
        bluetoothService,
        hfpProxyService,
        connectionFactory,
        configuration
    );
    app.start();

    let result = qApplication.exec();

    app.stop();

    std::for_each(threadPool.begin(), threadPool.end(), std::bind(&std::thread::join, std::placeholders::_1));

    libusb_exit(usbContext);

    return result;
}
