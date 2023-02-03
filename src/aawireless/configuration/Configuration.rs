use boost;
use std;

// #include <boost/property_tree/ptree.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include "Configuration.h"

pub fn Configuration(&file: std::string) {
    let iniConfig = boost::property_tree::ptree;
    boost::property_tree::ini_parser::read_ini(file, iniConfig);

    let wifiDevice = iniConfig.get<std::string>("Wifi.Device");
    let wifiIpAddress = iniConfig.get<std::string>("Wifi.IpAddress");
    let wifiPort = iniConfig.get<std::uint16_t>("Wifi.Port");
    let wifiSSID = iniConfig.get<std::string>("Wifi.SSID");
    //let wifiPassphrase = iniConfig.get<std::string>("Wifi.Passphrase"); // TODO: implement for hardcoded passphrase instead of generated
}
