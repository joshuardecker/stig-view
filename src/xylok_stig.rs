use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Anything commented out is present in a stig xml file,
// but not found in the xylok info.txt file.
#[derive(Debug, Default, PartialEq)]
pub struct Stig {
    //group_id: String,
    //title: String,
    pub version: String,

    //rule_id: String,
    //weight: String,
    //severity: String,
    pub introduction: String,
    pub description: String,
    pub check_text: String,
    pub fix_text: String,
}

#[derive(Debug)]
pub enum StigError {
    FileError,
    RegexError,
}

// Nicer printing for Stig.
impl std::fmt::Display for Stig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Version: {}\n\nIntro: {}\n\nDesc: {}\n\nCheck: {}\n\nFix: {}",
            self.version, self.introduction, self.description, self.check_text, self.fix_text
        )
    }
}

/// Load a stig file from the xylok format given a file path. Found as info.txt throughout xylok.
pub fn load_stig<P: AsRef<Path>>(path: P) -> Result<Stig, StigError> {
    let mut file = File::open(path).map_err(|_| StigError::FileError)?;

    let mut content = String::new();

    file.read_to_string(&mut content)
        .map_err(|_| StigError::FileError)?;

    // Set up the regex's.
    // Safe to call unwrap because these expressions are not runtime dependent.
    let re_version = Regex::new(r"# Title\n([a-zA-Z0-9-]*):").unwrap();
    let re_intro = Regex::new(r"(?s)# Title\n.*:(.*)#################\n# Similar checks").unwrap();
    let re_desc = Regex::new(r"(?s)# Discussion\n(.*)#################\n# Fix").unwrap();
    let re_check_text = Regex::new(r"(?s)# Content\n(.*)#################\n# Discussion").unwrap();
    let re_fix = Regex::new(r"(?s)# Fix\n(.*)").unwrap();

    // Capture regex content.
    let version_capture = re_version.captures(&content).ok_or(StigError::RegexError)?;
    let intro_capture = re_intro.captures(&content).ok_or(StigError::RegexError)?;
    let desc_capture = re_desc.captures(&content).ok_or(StigError::RegexError)?;
    let check_text_capture = re_check_text
        .captures(&content)
        .ok_or(StigError::RegexError)?;
    let fix_capture = re_fix.captures(&content).ok_or(StigError::RegexError)?;

    let mut stig = Stig::default();

    // Store captured data.
    // Use .trim() to remove extra \n the regex will capture.
    stig.version = version_capture[1].to_string();
    stig.introduction = intro_capture[1].trim().to_string();
    stig.description = desc_capture[1].trim().to_string();
    stig.check_text = check_text_capture[1].trim().to_string();
    stig.fix_text = fix_capture[1].trim().to_string();

    Ok(stig)
}

#[test]
// Test loading a xylok formatted stig.
fn test_loading_stig() {
    let stig = Stig {
        version: "CASA-FW-000260".to_string(),

        introduction: r"The Cisco ASA must be configured to forward management traffic to the Network Operations Center (NOC) via an IPsec tunnel.".to_string(),

        description: r"When the production network is managed in-band, the management network could be housed at a NOC that is located remotely at single or multiple interconnected sites. NOC interconnectivity, as well as connectivity between the NOC and the managed network, must be enabled using IPsec tunnels to provide the separation and integrity of the managed traffic.".to_string(),

        check_text: r"
Step 1: Verify that an IPsec crypto map has been configured and bound to the outside interface as shown in the example below.

crypto ipsec ikev1 transform-set IPSEC_TRANSFORM esp-aes-192 esp-sha-hmac
crypto map IPSEC_CRYPTO_MAP 1 match address MANAGEMENT_TRAFFIC
crypto map IPSEC_CRYPTO_MAP 1 set peer 10.3.1.1
crypto map IPSEC_CRYPTO_MAP 1 set ikev1 transform-set IPSEC_TRANSFORM
crypto map IPSEC_CRYPTO_MAP 1 set security-association lifetime seconds 3600
crypto map IPSEC_CRYPTO_MAP interface OUTSIDE

Step 2: Verify the there is a tunnel group configured for the peer defined in the crypto map as shown in the example below.

tunnel-group 10.3.1.1 type ipsec-l2l
tunnel-group 10.3.1.1 ipsec-attributes
 ikev1 pre-shared-key *****

Step 3: Verify that an ISAKMP policy for IKE connections has been configured and bound to the outside interface as shown in the example.

crypto isakmp identity address
crypto ikev1 enable OUTSIDE
crypto ikev1 policy 10
 authentication pre-share
 encryption aes-256
 hash sha
 group 5
 lifetime 3600

Step 4: Verify that the ACL referenced in the IPsec crypto map includes all applicable management traffic.

access-list MANAGEMENT_TRAFFIC extended permit udp any eq snmp 10.2.2.0 255.255.255.0
access-list MANAGEMENT_TRAFFIC extended permit udp any eq 10.2.2.0 255.255.255.0 snmptrap
access-list MANAGEMENT_TRAFFIC extended permit udp any eq syslog 10.2.2.0 255.255.255.0
access-list MANAGEMENT_TRAFFIC extended permit tcp any eq ssh 10.2.2.0 255.255.255.0

Note: Exception would be allowed for management traffic to and from managed perimeter devices.

If the ASA is not configured to forward management traffic to the NOC via an IPsec tunnel, this is a finding."
        .trim()
        .to_string(),

        fix_text: r"
Step 1: Configure an ISAKMP policy for IKE connection as shown in the example.

ASA1(config)# crypto ikev1 policy 10
ASA1(config-ikev1-policy)# authentication pre-share
ASA1(config-ikev1-policy)# encryption aes-256
ASA1(config-ikev1-policy)# hash sha
ASA1(config-ikev1-policy)# group 5
ASA1(config-ikev1-policy)# lifetime 3600
ASA1(config-ikev1-policy)# exit

Step 2: Enable the IKEv1 policy on the outside interface and identify itself with its IP address.

ASA1(config)# crypto ikev1 enable OUTSIDE
ASA1(config)# crypto isakmp identity address

Step 3: Configure the tunnel group as shown in the example below.

ASA2(config)# tunnel-group 10.10.10.1 ipsec-attributes
ASA2(config-tunnel-ipsec)# ikev1 pre-shared-key xxxxxxxxxxxxx

Step 4: Configure a transform set for encryption and authentication.

crypto ipsec ikev1 transform-set IPSEC_TRANSFORM esp-aes-192 esp-sha-hmac

Step 5: Configure the ACL to define the management traffic that will traverse the tunnel.

ASA1(config)# access-list MANAGEMENT_TRAFFIC extended permit udp any eq snmp 10.2.2.0 255.255.255.0
ASA1(config)# access-list MANAGEMENT_TRAFFIC extended permit udp any eq 10.2.2.0 255.255.255.0 snmptrap
ASA1(config)# access-list MANAGEMENT_TRAFFIC extended permit udp any eq syslog 10.2.2.0 255.255.255.0
ASA1(config)# access-list MANAGEMENT_TRAFFIC extended permit tcp any eq ssh 10.2.2.0 255.255.255.0

Step 6: Configure crypto map and bind to the outside interface as shown in the example below.

ASA1(config)# crypto map IPSEC_CRYPTO_MAP 1 match address MANAGEMENT_TRAFFIC
ASA1(config)# crypto map IPSEC_CRYPTO_MAP 1 set peer 10.10.10.2
ASA1(config)# crypto map IPSEC_CRYPTO_MAP 1 set ikev1 transform-set MY_TRANSFORM_SET
ASA1(config)# crypto map IPSEC_CRYPTO_MAP 1 set security-association lifetime seconds 3600
ASA1(config)# crypto map IPSEC_CRYPTO_MAP interface OUTSIDE"
        .trim()
        .to_string(),
    };

    let maybe_stig = load_stig("test.txt");

    match maybe_stig {
        Ok(loaded_stig) => {
            assert_eq!(
                loaded_stig, stig,
                "Stig loaded from file was not the same value as reference stig!"
            );
        }
        Err(e) => {
            panic!("Error: {:?} in function test_loading_stig!", e);
        }
    }
}
