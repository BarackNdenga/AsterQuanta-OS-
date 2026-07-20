SUMMARY = "aqm-supervisor - supervision reelle des services systemd + score de sante"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

inherit cargo systemd

SRC_URI = "file://Cargo.toml \
           file://src/main.rs \
           file://aqm-supervisor.service"

S = "${WORKDIR}"

SYSTEMD_SERVICE:${PN} = "aqm-supervisor.service"
SYSTEMD_AUTO_ENABLE:${PN} = "enable"

do_install:append() {
    install -d ${D}${systemd_unitdir}/system
    install -m 0644 ${WORKDIR}/aqm-supervisor.service ${D}${systemd_unitdir}/system/
    install -d ${D}${bindir}
    install -m 0755 ${B}/target/${RUST_HOST_SYS}/release/aqm-supervisor ${D}${bindir}/aqm-supervisor
}

FILES:${PN} += "${systemd_unitdir}/system/aqm-supervisor.service"
