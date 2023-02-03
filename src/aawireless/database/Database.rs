use boost;
use std;

// #include <boost/property_tree/ptree.hpp>
// #include <boost/filesystem.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include "Database.h"

struct DatabaseX {
    file: std::string,
    lastBluetoothDevice: std::string
}

impl DatabaseX {
    pub fn new(&self, &file: std::string) -> Self {
        Self{file: file}
    }
    pub fn load(&self) {
        let iniConfig: boost::property_tree::ptree;
        // TODO: create directories + file if not exist
        if (boost::filesystem::exists(self.file)) {
            boost::property_tree::ini_parser::read_ini(self.file, iniConfig);
        }
        let lastBluetoothDevice = iniConfig.get<std::string>("Bluetooth.LastDevice", std::string());
    }
    
    pub fn save(&self) {
        let iniConfig: boost::property_tree::ptree;
        if (boost::filesystem::exists(file)) {
            boost::property_tree::ini_parser::read_ini(self.file, iniConfig);
        }
        iniConfig.put("Bluetooth.LastDevice", lastBluetoothDevice);
        boost::property_tree::ini_parser::write_ini(self.file, iniConfig);
    }
    
    pub fn setLastBluetoothDevice(&mut self, address: std::string) {
        self.lastBluetoothDevice = address;
    }
    
    pub fn getLastBluetoothDevice(&self) {
        return self.lastBluetoothDevice;
    }
}
