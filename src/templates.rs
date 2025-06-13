use crate::voucher::VoucherManager;
use crate::wifi_network::WiFiNetwork;
use std::fs;

// Load template files at compile time or runtime
fn load_template(name: &str) -> String {
    let template_path = format!("templates/{}.html", name);
    fs::read_to_string(&template_path)
        .unwrap_or_else(|_| panic!("Failed to load template: {}", template_path))
}

pub fn index_template() -> String {
    load_template("index")
}

pub fn no_vouchers_template() -> String {
    load_template("no-vouchers")
}

pub fn voucher_card_template() -> String {
    load_template("voucher-card")
}

pub fn vouchers_template() -> String {
    load_template("vouchers")
}

pub fn generate_voucher_card(qr_code_base64: &str, network_ssid: &str, voucher_code: &str) -> String {
    voucher_card_template()
        .replace("{{QR_CODE_BASE64}}", qr_code_base64)
        .replace("{{NETWORK_SSID}}", network_ssid)
        .replace("{{VOUCHER_CODE}}", voucher_code)
}

pub fn generate_vouchers_page(
    voucher_count: usize,
    network_name: &str,
    network_ssid: &str,
    voucher_cards: &str,
) -> String {
    vouchers_template()
        .replace("{{VOUCHER_COUNT}}", &voucher_count.to_string())
        .replace("{{NETWORK_NAME}}", network_name)
        .replace("{{NETWORK_SSID}}", network_ssid)
        .replace("{{VOUCHER_CARDS}}", voucher_cards)
}

pub fn admin_template(networks: &[&WiFiNetwork], voucher_manager: &VoucherManager) -> String {
    let template = load_template("admin");
    
    let network_rows = networks
        .iter()
        .map(|network| {
            let voucher_count = voucher_manager.voucher_count_for_network(&network.id);
            let unused_count = voucher_manager.unused_voucher_count_for_network(&network.id);
            let status_badge = if network.is_active {
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">Active</span>"#
            } else {
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">Inactive</span>"#
            };
            
            format!(
                r#"
                <tr class="hover:bg-gray-50">
                    <td class="px-6 py-4 whitespace-nowrap">
                        <div class="text-sm font-medium text-gray-900">{}</div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                        <div class="text-sm font-mono text-gray-500 bg-gray-100 px-2 py-1 rounded">{}</div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                        {}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                        <div class="text-sm text-gray-900">{}</div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                        <div class="text-sm text-gray-900">{} / {}</div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                        <div class="flex space-x-2">
                            <a href="/admin/networks/{}/vouchers" class="bg-blue-100 hover:bg-blue-200 text-blue-800 px-3 py-1 rounded-md text-xs font-medium transition-colors duration-200">
                                <i class="fas fa-list mr-1"></i>View
                            </a>
                            <a href="/generate?network_id={}" class="bg-green-100 hover:bg-green-200 text-green-800 px-3 py-1 rounded-md text-xs font-medium transition-colors duration-200">
                                <i class="fas fa-print mr-1"></i>Generate
                            </a>
                            <form method="post" action="/admin/networks/{}/delete" class="inline">
                                <button type="submit" class="bg-red-100 hover:bg-red-200 text-red-800 px-3 py-1 rounded-md text-xs font-medium transition-colors duration-200" onclick="return confirm('Are you sure?')">
                                    <i class="fas fa-trash mr-1"></i>Delete
                                </button>
                            </form>
                        </div>
                    </td>
                </tr>
                "#,
                network.name,
                network.ssid,
                status_badge,
                network.description.as_ref().unwrap_or(&"".to_string()),
                unused_count,
                voucher_count,
                network.id,
                network.id,
                network.id
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let network_options = networks
        .iter()
        .map(|network| {
            format!(
                r#"<option value="{}">{} ({})</option>"#,
                network.id, network.name, network.ssid
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let empty_networks_message = if networks.is_empty() {
        r#"<div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
            <div class="flex items-center">
                <i class="fas fa-info-circle text-blue-500 mr-2"></i>
                <span class="text-blue-800">No networks created yet. Create your first network above.</span>
            </div>
        </div>"#
    } else {
        ""
    };

    template
        .replace("{{NETWORK_OPTIONS}}", &network_options)
        .replace("{{EMPTY_NETWORKS_MESSAGE}}", empty_networks_message)
        .replace("{{NETWORK_ROWS}}", &network_rows)
}

pub fn network_vouchers_template(
    network: Option<&WiFiNetwork>,
    vouchers: &[&crate::voucher::Voucher],
    network_id: &str,
) -> String {
    let template = load_template("network-vouchers");
    
    let network_info = match network {
        Some(net) => format!(
            r#"
            <div class="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-8">
                <h5 class="text-xl font-semibold text-blue-900 mb-3">
                    <i class="fas fa-wifi mr-2"></i>{}
                </h5>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div>
                        <span class="font-medium text-blue-800">SSID:</span>
                        <span class="ml-2 font-mono bg-blue-100 px-2 py-1 rounded text-blue-900">{}</span>
                    </div>
                    <div>
                        <span class="font-medium text-blue-800">Description:</span>
                        <span class="ml-2 text-blue-700">{}</span>
                    </div>
                </div>
            </div>
            "#,
            net.name,
            net.ssid,
            net.description.as_ref().unwrap_or(&"No description".to_string())
        ),
        None => r#"
            <div class="bg-red-50 border border-red-200 rounded-lg p-6 mb-8">
                <h5 class="text-xl font-semibold text-red-900 mb-2">
                    <i class="fas fa-exclamation-triangle mr-2"></i>Network Not Found
                </h5>
                <p class="text-red-700">The requested network could not be found.</p>
            </div>
        "#.to_string(),
    };

    let voucher_rows = vouchers
        .iter()
        .enumerate()
        .map(|(i, voucher)| {
            let status_badge = if voucher.is_used {
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">Used</span>"#
            } else {
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">Available</span>"#
            };
            
            format!(
                r#"
                <tr class="hover:bg-gray-50">
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{}</td>
                    <td class="px-6 py-4 whitespace-nowrap">
                        <div class="text-sm font-mono bg-gray-100 px-2 py-1 rounded text-gray-900">{}</div>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">{}</td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{}</td>
                </tr>
                "#,
                i + 1,
                voucher.code,
                status_badge,
                voucher.created_at.format("%Y-%m-%d %H:%M")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let empty_vouchers_message = if vouchers.is_empty() {
        r#"<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
            <div class="flex items-center">
                <i class="fas fa-exclamation-triangle text-yellow-500 mr-2"></i>
                <span class="text-yellow-800">No vouchers found for this network.</span>
            </div>
        </div>"#
    } else {
        ""
    };

    template
        .replace("{{NETWORK_ID}}", network_id)
        .replace("{{NETWORK_INFO}}", &network_info)
        .replace("{{VOUCHER_COUNT}}", &vouchers.len().to_string())
        .replace("{{EMPTY_VOUCHERS_MESSAGE}}", empty_vouchers_message)
        .replace("{{VOUCHER_ROWS}}", &voucher_rows)
}

