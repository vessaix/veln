PORTNAME=	veln
DISTVERSION=	0.1.0
CATEGORIES=	sysutils

MAINTAINER=	amr@vessaix.com
COMMENT=	FreeBSD Virtualization Management CLI
WWW=		https://github.com/vessaix/veln

LICENSE=	BSD3CLAUSE
LICENSE_FILE=	${WRKSRC}/LICENSE

USES=		cargo
USE_GITHUB=	yes
GH_ACCOUNT=	vessaix

PLIST_FILES=	bin/veln

OPTIONS_DEFINE=	API LOCAL_SOURCE
OPTIONS_DEFAULT=API
API_DESC=	Install RC script for API server
LOCAL_SOURCE_DESC=Build from local source (no fetch)

API_PLIST_FILES=	etc/rc.d/veln
API_VARS=		USE_RC_SUBR=veln

# Local source support
.if ${PORT_OPTIONS:MLOCAL_SOURCE}
# Override to use local source tree
WRKSRC=		${LOCALBASE}/veln-src
DISTFILES=	
NO_FETCH=	yes
.endif

.include <bsd.port.pre.mk>

.if ${PORT_OPTIONS:MLOCAL_SOURCE}
pre-fetch:
	@if [ ! -f "${WRKSRC}/Cargo.toml" ]; then \
		echo "Error: Local source not found at ${WRKSRC}"; \
		echo "Set WRKSRC to your local veln source directory"; \
		exit 1; \
	fi
	@${ECHO_MSG} "Building from local source: ${WRKSRC}"
.endif

.include <bsd.port.post.mk>
