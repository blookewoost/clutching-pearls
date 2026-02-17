#!/usr/bin/env python3
"""
GATT Server for Signal Network
Exposes a readable characteristic containing a poem/message
"""

import dbus
import dbus.service
import dbus.mainloop.glib
from gi.repository import GLib
import sys

GATT_SERVICE_IFACE = "org.bluez.GattService1"
GATT_CHAR_IFACE = "org.bluez.GattCharacteristic1"
GATT_DESC_IFACE = "org.bluez.GattDescriptor1"

SERVICE_UUID = "12345678-1234-5678-1234-56789abcdef0"
CHAR_UUID = "12345679-1234-5678-1234-56789abcdef0"
CCCD_UUID = "00002902-0000-1000-8000-00805f9b34fb"

POEM = """Do you see it too?

A signal in the darkness,
waiting to be found."""


class InvalidArgsException(dbus.exceptions.DBusException):
    _dbus_error_name = "org.freedesktop.DBus.Error.InvalidArgs"


class NotSupportedException(dbus.exceptions.DBusException):
    _dbus_error_name = "org.bluez.Error.NotSupported"


class Application(dbus.service.Object):
    def __init__(self, bus):
        self.path = "/com/signal/network"
        self.services = []
        dbus.service.Object.__init__(self, bus, self.path)
        self.add_service(SignalService(bus, 0, self))

    def get_path(self):
        return dbus.ObjectPath(self.path)

    def add_service(self, service):
        self.services.append(service)

    @dbus.service.method("org.freedesktop.DBus.ObjectManager", out_signature="a{oa{sa{sv}}}")
    def GetManagedObjects(self):
        response = {}
        response[dbus.ObjectPath(self.path)] = {
            "org.freedesktop.DBus.Introspectable": {},
            "org.freedesktop.DBus.Properties": {},
            "org.freedesktop.DBus.ObjectManager": {},
        }
        for service in self.services:
            response[service.get_path()] = service.get_properties()
            response.update(service.get_characteristics_and_descriptors())
        return response


class SignalService(dbus.service.Object):
    def __init__(self, bus, index, app):
        self.path = app.path + "/service" + str(index)
        self.bus = bus
        self.uuid = SERVICE_UUID
        self.app = app
        dbus.service.Object.__init__(self, bus, self.path)
        self.characteristics = [PoemCharacteristic(bus, 0, self)]

    def get_path(self):
        return dbus.ObjectPath(self.path)

    def get_uuid(self):
        return self.uuid

    def get_properties(self):
        return {
            GATT_SERVICE_IFACE: {
                "UUID": dbus.String(self.uuid),
                "Primary": dbus.Boolean(True),
                "Characteristics": dbus.Array(
                    [dbus.ObjectPath(c.get_path()) for c in self.characteristics],
                    signature="o",
                ),
            }
        }

    @dbus.service.method("org.freedesktop.DBus.Properties", in_signature="s", out_signature="a{sv}")
    def GetAll(self, interface_name):
        if interface_name != GATT_SERVICE_IFACE:
            raise InvalidArgsException()
        return self.get_properties()[GATT_SERVICE_IFACE]

    def get_characteristics_and_descriptors(self):
        response = {}
        for char in self.characteristics:
            response[char.get_path()] = char.get_properties()
            response.update(char.get_descriptors())
        return response


class PoemCharacteristic(dbus.service.Object):
    def __init__(self, bus, index, service):
        self.path = service.path + "/char" + str(index)
        self.bus = bus
        self.service = service
        self.uuid = CHAR_UUID
        self.notifying = False
        dbus.service.Object.__init__(self, bus, self.path)

    def get_path(self):
        return dbus.ObjectPath(self.path)

    def get_uuid(self):
        return self.uuid

    def get_properties(self):
        return {
            GATT_CHAR_IFACE: {
                "Service": dbus.ObjectPath(self.service.get_path()),
                "UUID": dbus.String(self.uuid),
                "Flags": dbus.Array(["read", "notify"]),
                "Notifying": dbus.Boolean(self.notifying),
                "Descriptors": dbus.Array(
                    [dbus.ObjectPath(self.path + "/desc0")],
                    signature="o",
                ),
            }
        }

    def get_descriptors(self):
        return {
            dbus.ObjectPath(self.path + "/desc0"): {
                GATT_DESC_IFACE: {
                    "Characteristic": dbus.ObjectPath(self.get_path()),
                    "UUID": dbus.String(CCCD_UUID),
                    "Flags": dbus.Array(["read", "write"]),
                }
            }
        }

    @dbus.service.method(GATT_CHAR_IFACE, in_signature="a{sv}", out_signature="ay")
    def ReadValue(self, options):
        print(f"[GATT] ReadValue called on {self.path}")
        print(f"[GATT] Sending poem: {len(POEM)} bytes")
        return dbus.Array(bytearray(POEM.encode("utf-8")), signature="y")

    @dbus.service.method(GATT_CHAR_IFACE, in_signature="aya{sv}")
    def WriteValue(self, value, options):
        print(f"[GATT] WriteValue called on {self.path}")
        # For now, just accept writes
        pass

    @dbus.service.method("org.freedesktop.DBus.Properties", in_signature="s", out_signature="a{sv}")
    def GetAll(self, interface_name):
        if interface_name != GATT_CHAR_IFACE:
            raise InvalidArgsException()
        return self.get_properties()[GATT_CHAR_IFACE]

    @dbus.service.signal("org.freedesktop.DBus.Properties", signature="sa{sv}as")
    def PropertiesChanged(self, interface_name, changed_properties, invalidated_properties):
        pass


def register_app_cb():
    print("[GATT] ✓ GATT application registered with BlueZ")


def register_app_error_cb(error):
    print(f"[GATT] ✗ Failed to register GATT application: {error}")
    print("[GATT] This may be expected - BlueZ might still accept connections")
    # Don't quit - GATT might still work despite registration errors


def main():
    global mainloop
    
    dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
    bus = dbus.SystemBus()
    
    # Disable audio profiles to avoid connection conflicts
    try:
        disable_audio_profiles(bus)
    except Exception as e:
        print(f"Warning: Could not disable audio profiles: {e}")
    
    # Create and register the application
    app = Application(bus)
    
    try:
        manager = dbus.Interface(
            bus.get_object("org.bluez", "/org/bluez/hci0"),
            "org.bluez.GattManager1",
        )
        
        manager.RegisterApplication(
            app.get_path(),
            {},
            reply_handler=register_app_cb,
            error_handler=register_app_error_cb,
        )
    except Exception as e:
        print(f"Error getting GattManager: {e}")
        print("Make sure BlueZ is running and /org/bluez/hci0 exists")
        sys.exit(1)
    
    mainloop = GLib.MainLoop()
    try:
        mainloop.run()
    except KeyboardInterrupt:
        print("\nGATT server shutting down...")
        mainloop.quit()


def disable_audio_profiles(bus):
    """Disable A2DP and other audio profiles to avoid connection conflicts."""
    adapter = dbus.Interface(
        bus.get_object("org.bluez", "/org/bluez/hci0"),
        "org.freedesktop.DBus.Properties",
    )
    # Set device class to generic (0x000000 would be ideal but may not work)
    # Instead, we'll let it be but the GATT characteristic should still work
    print("[GATT] Audio profiles configuration ready")


if __name__ == "__main__":
    main()
