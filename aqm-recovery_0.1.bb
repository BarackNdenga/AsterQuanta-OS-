SUMMARY = "aqm-recovery - snapshots BTRFS, mode sur, cible de recovery"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

SRC_URI = "file://aqm-recovery.sh \
           file://aqm-recovery.service \
           file://aqm-recovery.target \
           file://aqm-safe.target"

S = "${WORKDIR}"

RDEPENDS:${PN} = "btrfs-tools bash"

inherit systemd

SYSTEMD_SERVICE:${PN} = "aqm-recovery.service"
SYSTEMD_AUTO_ENABLE:${PN} = "enable"

do_install() {
    install -d ${D}${bindir}
    install -m 0755 ${WORKDIR}/aqm-recovery.sh ${D}${bindir}/aqm-recovery
    install -d ${D}${systemd_unitdir}/system
    install -m 0644 ${WORKDIR}/aqm-recovery.service ${D}${systemd_unitdir}/system/
    install -m 0644 ${WORKDIR}/aqm-recovery.target ${D}${systemd_unitdir}/system/
    install -m 0644 ${WORKDIR}/aqm-safe.target ${D}${systemd_unitdir}/system/
}

FILES:${PN} += "${systemd_unitdir}/system/aqm-recovery.service \
                ${systemd_unitdir}/system/aqm-recovery.target \
                ${systemd_unitdir}/system/aqm-safe.target"
