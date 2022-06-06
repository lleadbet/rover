use rover_client::operations::subgraph::async_check::{self, SubgraphCheckAsyncInput};
use serde::Serialize;
use structopt::StructOpt;

use rover_client::operations::subgraph::check::{self, SubgraphCheckInput};
use rover_client::shared::{CheckConfig, GitContext, GraphRef, ValidationPeriod};

use crate::command::RoverOutput;
use crate::utils::client::StudioClientConfig;
use crate::utils::parsers::{
    parse_file_descriptor, parse_query_count_threshold, parse_query_percentage_threshold,
    FileDescriptorType,
};
use crate::Result;

#[derive(Debug, Serialize, StructOpt)]
pub struct Check {
    /// <NAME>@<VARIANT> of graph in Apollo Studio to validate.
    /// @<VARIANT> may be left off, defaulting to @current
    #[structopt(name = "GRAPH_REF")]
    #[serde(skip_serializing)]
    graph: GraphRef,

    /// Name of the subgraph to validate
    #[structopt(long = "name")]
    #[serde(skip_serializing)]
    subgraph: String,

    /// Name of configuration profile to use
    #[structopt(long = "profile", default_value = "default")]
    #[serde(skip_serializing)]
    profile_name: String,

    /// The schema file to check. You can pass `-` to use stdin instead of a file.
    #[structopt(long, short = "s", parse(try_from_str = parse_file_descriptor))]
    #[serde(skip_serializing)]
    schema: FileDescriptorType,

    /// The minimum number of times a query or mutation must have been executed
    /// in order to be considered in the check operation
    #[structopt(long, parse(try_from_str = parse_query_count_threshold))]
    query_count_threshold: Option<i64>,

    /// Minimum percentage of times a query or mutation must have been executed
    /// in the time window, relative to total request count, for it to be
    /// considered in the check. Valid numbers are in the range 0 <= x <= 100
    #[structopt(long, parse(try_from_str = parse_query_percentage_threshold))]
    query_percentage_threshold: Option<f64>,

    /// Size of the time window with which to validate schema against (i.e "24h" or "1w 2d 5h")
    #[structopt(long)]
    validation_period: Option<ValidationPeriod>,

    /// If the check should be run asynchronously
    #[structopt(long = "async", short = "a")]
    asynchronous: bool,
}

impl Check {
    pub fn run(
        &self,
        client_config: StudioClientConfig,
        git_context: GitContext,
    ) -> Result<RoverOutput> {
        let client = client_config.get_authenticated_client(&self.profile_name)?;

        let proposed_schema = self
            .schema
            .read_file_descriptor("SDL", &mut std::io::stdin())?;

        eprintln!(
            "Checking the proposed schema for subgraph {} against {}",
            &self.subgraph, &self.graph
        );

        if self.asynchronous {
            let res = async_check::run(
                SubgraphCheckAsyncInput {
                    graph_ref: self.graph.clone(),
                    subgraph: self.subgraph.clone(),
                    git_context,
                    proposed_schema,
                    config: CheckConfig {
                        query_count_threshold: self.query_count_threshold,
                        query_count_threshold_percentage: self.query_percentage_threshold,
                        validation_period: self.validation_period.clone(),
                    },
                },
                &client,
            )?;

            Ok(RoverOutput::AsyncCheckResponse(res))
        } else {
            let res = check::run(
                SubgraphCheckInput {
                    graph_ref: self.graph.clone(),
                    proposed_schema,
                    subgraph: self.subgraph.clone(),
                    git_context,
                    config: CheckConfig {
                        query_count_threshold: self.query_count_threshold,
                        query_count_threshold_percentage: self.query_percentage_threshold,
                        validation_period: self.validation_period.clone(),
                    },
                },
                &client,
            )?;
            Ok(RoverOutput::CheckResponse(res))
        }
    }
}
