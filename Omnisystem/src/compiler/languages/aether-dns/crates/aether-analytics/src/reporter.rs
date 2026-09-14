/// Analytics Reporter
/// Report generation and export

use crate::aggregator::MetricsAggregator;
use crate::dashboard::DashboardData;
use serde::{Deserialize, Serialize};

pub struct AnalyticsReporter {
    aggregator: MetricsAggregator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub report_type: String,
    pub generated_at: String,
    pub period: String,
    pub sections: Vec<ReportSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub data: String, // JSON serialized data
}

impl AnalyticsReporter {
    pub fn new(aggregator: MetricsAggregator) -> Self {
        AnalyticsReporter { aggregator }
    }

    pub fn generate_summary_report(&self) -> AnalyticsReport {
        let agg_metrics = self.aggregator.get_aggregated_metrics();
        let top_domains = self.aggregator.get_top_domains(10);
        let top_sources = self.aggregator.get_top_sources(10);
        let threats = self.aggregator.get_threat_distribution();

        let dashboard = DashboardData::new(&agg_metrics, top_domains, top_sources, threats);

        let mut sections = Vec::new();

        sections.push(ReportSection {
            title: "Summary Statistics".to_string(),
            data: serde_json::to_string(&dashboard.summary).unwrap_or_default(),
        });

        sections.push(ReportSection {
            title: "Performance Metrics".to_string(),
            data: serde_json::to_string(&dashboard.performance).unwrap_or_default(),
        });

        sections.push(ReportSection {
            title: "Security Analysis".to_string(),
            data: serde_json::to_string(&dashboard.security).unwrap_or_default(),
        });

        AnalyticsReport {
            report_type: "Summary Report".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            period: "Last 24 Hours".to_string(),
            sections,
        }
    }

    pub fn generate_security_report(&self) -> AnalyticsReport {
        let threats = self.aggregator.get_threat_distribution();

        let mut sections = Vec::new();

        sections.push(ReportSection {
            title: "Threat Distribution".to_string(),
            data: serde_json::to_string(&threats).unwrap_or_default(),
        });

        sections.push(ReportSection {
            title: "Top Threat Sources".to_string(),
            data: serde_json::to_string(&self.aggregator.get_top_sources(20))
                .unwrap_or_default(),
        });

        AnalyticsReport {
            report_type: "Security Report".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            period: "Last 24 Hours".to_string(),
            sections,
        }
    }

    pub fn export_csv(&self, filename: &str) -> anyhow::Result<()> {
        let domains = self.aggregator.get_domain_stats();
        let mut csv_content = String::from("domain,query_count\n");

        for (domain, count) in domains {
            csv_content.push_str(&format!("{},{}\n", domain, count));
        }

        std::fs::write(filename, csv_content)?;
        Ok(())
    }

    pub fn get_dashboard_data(&self) -> DashboardData {
        let agg = self.aggregator.get_aggregated_metrics();
        let top_domains = self.aggregator.get_top_domains(10);
        let top_sources = self.aggregator.get_top_sources(10);
        let threats = self.aggregator.get_threat_distribution();

        DashboardData::new(&agg, top_domains, top_sources, threats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_creation() {
        let agg = MetricsAggregator::new();
        let reporter = AnalyticsReporter::new(agg);
        let report = reporter.generate_summary_report();
        assert_eq!(report.report_type, "Summary Report");
    }

    #[test]
    fn test_security_report() {
        let agg = MetricsAggregator::new();
        let reporter = AnalyticsReporter::new(agg);
        let report = reporter.generate_security_report();
        assert_eq!(report.report_type, "Security Report");
    }
}
