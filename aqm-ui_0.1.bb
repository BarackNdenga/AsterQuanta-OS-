SUMMARY = "aqm-ui - interface graphique kiosque lancee automatiquement au boot"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

DEPENDS = "gtk4 glib-2.0"
RDEPENDS:${PN} = "cage aqm-shell"

inherit systemd pkgconfig

SRC_URI = "file://main.c \
           file://meson.build \
           file://aqm-ui.service"

S = "${WORKDIR}"

inherit meson

SYSTEMD_SERVICE:${PN} = "aqm-ui.service"
SYSTEMD_AUTO_ENABLE:${PN} = "enable"

do_install:append() {
    install -d ${D}${systemd_unitdir}/system
    install -m 0644 ${WORKDIR}/aqm-ui.service ${D}${systemd_unitdir}/system/
}

FILES:${PN} += "${systemd_unitdir}/system/aqm-ui.service"
