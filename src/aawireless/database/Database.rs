use boost;
use std;

// #include <boost/property_tree/ptree.hpp>
// #include <boost/filesystem.hpp>
// #include <boost/property_tree/ini_parser.hpp>
// #include "Database.h"

pub struct DatabaseX {
    file: String,
    lastBluetoothDevice: String
}

impl DatabaseX {
    pub fn new(&self, &file: String) -> Self {
        Self{file: file, lastBluetoothDevice: ' '}
    }
    pub fn load(&self) {
        let iniConfig: boost::property_tree::ptree;
        // TODO: create directories + file if not exist
        if (boost::filesystem::exists(self.file)) {
            boost::property_tree::ini_parser::read_ini(self.file, iniConfig);
        }
        self.lastBluetoothDevice = iniConfig.get<String>("Bluetooth.LastDevice", String());
    }
    
    pub fn save(&self) {
        let iniConfig: boost::property_tree::ptree;
        if (boost::filesystem::exists(self.file)) {
            boost::property_tree::ini_parser::read_ini(self.file, iniConfig);
        }
        iniConfig.put("Bluetooth.LastDevice", self.lastBluetoothDevice);
        boost::property_tree::ini_parser::write_ini(self.file, iniConfig);
    }
    
    pub fn setLastBluetoothDevice(&mut self, address: String) {
        self.lastBluetoothDevice = address;
    }
    
    pub fn getLastBluetoothDevice(&self) {
        return self.lastBluetoothDevice;
    }
}
