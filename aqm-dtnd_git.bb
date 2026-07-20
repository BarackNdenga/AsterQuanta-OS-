SUMMARY = "aqm-dtnd - noeud DTN (Bundle Protocol, RFC 9171) via uD3TN upstream"
DESCRIPTION = "Wrapper Yocto autour d'uD3TN, implementation reelle et \
maintenue du Bundle Protocol pour environnements a connectivite \
intermittente (utilisee dans de vrais projets spatiaux/IoT). Pas de \
protocole reinvente."
HOMEPAGE = "https://gitlab.com/d3tn/ud3tn"
LICENSE = "BSD-3-Clause"
# Le md5 exact depend du commit fetche (SRCREV = AUTOREV). Au premier
# `kas build`, bitbake echouera avec le vrai md5 attendu dans son message
# d'erreur "Fetcher failure ... file was expected to be a checksum of" —
# copier cette valeur ici. Ne pas builder AUTOREV en production: figer
# SRCREV sur un hash de commit precis une fois la version validee.
LIC_FILES_CHKSUM = "file://LICENSE;md5=TO_BE_FILLED_ON_FIRST_FETCH"

SRC_URI = "git://gitlab.com/d3tn/ud3tn.git;protocol=https;branch=master \
           file://aqm-dtnd.service"
SRCREV = "${AUTOREV}"

S = "${WORKDIR}/git"

DEPENDS = "libcbor mbedtls"

inherit systemd

SYSTEMD_SERVICE:${PN} = "aqm-dtnd.service"
SYSTEMD_AUTO_ENABLE:${PN} = "enable"

EXTRA_OEMAKE = "TYPE=release"

do_compile() {
    oe_runmake -C ${S} posix
}

do_install() {
    install -d ${D}${bindir}
    install -m 0755 ${S}/build/posix/ud3tn ${D}${bindir}/aqm-dtnd
    install -d ${D}${systemd_unitdir}/system
    install -m 0644 ${WORKDIR}/aqm-dtnd.service ${D}${systemd_unitdir}/system/
}

FILES:${PN} += "${systemd_unitdir}/system/aqm-dtnd.service"
