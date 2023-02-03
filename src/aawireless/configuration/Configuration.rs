use boost;
use std;

// #include <boost/property_tree/ptree.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include "Configuration.h"

struct Configuration {
    wifiDevice: std::string,
    wifiIpAddress: std::string,
    wifiPort: uint16_t,
    wifiBSSID: std::string,
    wifiSSID: std::string,
    wifiPassphrase: std::string,
    wifiHotspotScript: std::string,
}
impl Configuration {
    pub fn new(&mut self, &file: std::string) -> Self {
        let iniConfig = boost::property_tree::ptree;
        boost::property_tree::ini_parser::read_ini(file, iniConfig);
    
        return Self {
            wifiDevice: iniConfig.get<std::string>("Wifi.Device"),
            wifiIpAddress: iniConfig.get<std::string>("Wifi.IpAddress"),
            wifiPort: iniConfig.get<std::uint16_t>("Wifi.Port"),
            wifiSSID: iniConfig.get<std::string>("Wifi.SSID"),
        }
        //wifiPassphrase: iniConfig.get<std::string>("Wifi.Passphrase"); // TODO: implement for hardcoded passphrase instead of generated
    }
}
