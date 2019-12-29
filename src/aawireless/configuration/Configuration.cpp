//
// Created by chiel on 29-12-19.
//

#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/ini_parser.hpp>
#include "Configuration.h"

namespace aawireless {
    namespace configuration {
        Configuration::Configuration(const std::string &file) {
            boost::property_tree::ptree iniConfig;
            boost::property_tree::ini_parser::read_ini(file, iniConfig);

            wifiIpAddress = iniConfig.get<std::string>("Wifi.IpAddress");
            wifiPort = iniConfig.get<std::uint16_t>("Wifi.Port");
            wifiBSSID = iniConfig.get<std::string>("Wifi.BSSID");
            wifiSSID = iniConfig.get<std::string>("Wifi.SSID");
            wifiPassphrase = iniConfig.get<std::string>("Wifi.Passphrase");
        }
    }
}
