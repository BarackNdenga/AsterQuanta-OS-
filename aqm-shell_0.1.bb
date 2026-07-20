SUMMARY = "aqm-shell (aqmctl) - CLI d'administration AsterQuanta OS"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

inherit cargo

SRC_URI = "file://Cargo.toml \
           file://src/main.rs"

S = "${WORKDIR}"

do_install:append() {
    install -d ${D}${bindir}
    install -m 0755 ${B}/target/${RUST_HOST_SYS}/release/aqmctl ${D}${bindir}/aqmctl
}

RDEPENDS:${PN} += "aqm-supervisor rauc systemd"
