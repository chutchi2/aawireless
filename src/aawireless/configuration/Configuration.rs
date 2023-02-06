use boost;
use std;

// #include <boost/property_tree/ptree.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include "Configuration.h"

pub struct Configuration {
    pub wifiDevice: String,
    pub wifiIpAddress: String,
    pub wifiPort: u16,
    pub wifiBSSID: String,
    pub wifiSSID: String,
    pub wifiPassphrase: String,
    wifiHotspotScript: String,
}
impl Configuration {
    pub fn new(&mut self, &file: String) -> Self {
        let iniConfig = boost::property_tree::ptree;
        boost::property_tree::ini_parser::read_ini(file, iniConfig);
    
        return Self {
            wifiDevice: iniConfig.get<String>("Wifi.Device"),
            wifiIpAddress: iniConfig.get<String>("Wifi.IpAddress"),
            wifiPort: iniConfig.get<u16>("Wifi.Port"),
            wifiSSID: iniConfig.get<String>("Wifi.SSID"),
            wifiBSSID: iniConfig.get<String>("Wifi.BSSID"),
            wifiPassphrase: iniConfig.get<String>("Wifi.Passphrase"),
            wifiHotspotScript: iniConfig.get<String>("Wifi.HotspotScript"),
        }
        //wifiPassphrase: iniConfig.get<String>("Wifi.Passphrase"); // TODO: implement for hardcoded passphrase instead of generated
    }
}
