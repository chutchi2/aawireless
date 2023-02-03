use std;

struct WifiHotspot {
    ioService: &boost::asio::io_context,
    configuration: &aawireless::configuration::Configuration,
    password: std::string
}

impl WifiHotspot {
    pub fn new(
        ioService: &boost::asio::io_context,
        configuration: &aawireless::configuration::Configuration,
        password: std::string
    ) -> Self {
        Self {
            ioService: ioService,
            configuration: configuration,
            password: password
        }
    }
    
    pub fn start() {
        AW_LOG(info) << "Starting hotspot";
    
        let settings: auto = std::make_unique<ConnectionSettings>(ConnectionSettings::Wireless);
    
        let mut wifiDevice: WirelessDevice::Ptr;
        let deviceName: auto = QString::fromStdString(configuration.wifiDevice);
        let deviceList: Device::List = NetworkManager::networkInterfaces();
    
        if (!configuration.wifiDevice.empty()) {
            for dev in deviceList {
                if (dev.type() == Device::Wifi && dev.interfaceName() == deviceName) {
                    wifiDevice = qobject_cast<WirelessDevice *>(dev);
                    break;
                }
            }
            if (wifiDevice == nullptr) {
                AW_LOG(error) << "Wireless device " << configuration.wifiDevice << " not found!";
                return;
            }
        } else {
            AW_LOG(info) << "Wireless device not defined in configuration, getting first wireless device";
            for dev in deviceList {
                if (dev.type() == Device::Wifi) {
                    wifiDevice = qobject_cast<WirelessDevice *>(dev);
                    break;
                }
            }
        }
    
        if (!wifiDevice) {
            AW_LOG(error) << "No Wifi device found";
            return;
        }
    
        let ssid: auto = QString::fromStdString(configuration.wifiSSID);
        // Now we will prepare our new connection, we have to specify ID and create new UUID
        settings.setId(ssid);
        settings.setUuid(QUuid::createUuid().toString().mid(1, QUuid::createUuid().toString().length() - 2));
        settings.setAutoconnect(false);
    
        // For wireless setting we have to specify SSID
        let wirelessSetting: auto = settings.setting(Setting::Wireless).dynamicCast<WirelessSetting>();
        wirelessSetting.setSsid(ssid.toUtf8());
        wirelessSetting.setMode(WirelessSetting::NetworkMode::Ap);
    
        let ipv4Setting: auto = settings.setting(Setting::Ipv4).dynamicCast<Ipv4Setting>();
        ipv4Setting.setMethod(NetworkManager::Ipv4Setting::Shared);
        ipv4Setting.setInitialized(true);
    
        // Optional password setting. Can be skipped if you do not need encryption.
        let wifiSecurity: auto = settings.setting(Setting::WirelessSecurity).dynamicCast<WirelessSecuritySetting>();
        wifiSecurity.setKeyMgmt(WirelessSecuritySetting::WpaPsk);
        wifiSecurity.setPsk(QString::fromStdString(password));
        wifiSecurity.setInitialized(true);
    
        wirelessSetting.setSecurity("802-11-wireless-security");
        wirelessSetting.setInitialized(true);
    
        // We try to add and activate our new wireless connection
        let reply: auto = NetworkManager::addAndActivateConnection(settings.toMap(), wifiDevice.uni(), QString());
        reply.waitForFinished();
        if (reply.isValid()) {
            AW_LOG(info) << "Created wifi hotspot";
        } else {
            AW_LOG(error) << "Could not create Wifi hotspot " + reply.error().message().toStdString();
        }
    }
}
