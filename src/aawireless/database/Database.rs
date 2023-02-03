use boost;
use std;

// #include <boost/property_tree/ptree.hpp>
// #include <boost/filesystem.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include "Database.h"

pub fn Database(&file: std::string) -> file(file) {
    load();
}

pub fn load() {
    let iniConfig: boost::property_tree::ptree;
    // TODO: create directories + file if not exist
    if (boost::filesystem::exists(file)) {
        boost::property_tree::ini_parser::read_ini(file, iniConfig);
    }
    let lastBluetoothDevice = iniConfig.get<std::string>("Bluetooth.LastDevice", std::string());
}

pub fn save() {
    let iniConfig: boost::property_tree::ptree;
    if (boost::filesystem::exists(file)) {
        boost::property_tree::ini_parser::read_ini(file, iniConfig);
    }
    iniConfig.put("Bluetooth.LastDevice", lastBluetoothDevice);
    let boost::property_tree::ini_parser::write_ini(file, iniConfig);
}

pub fn setLastBluetoothDevice(address: std::string) {
    let lastBluetoothDevice = address;
}

pub fn getLastBluetoothDevice() {
    return lastBluetoothDevice;
}