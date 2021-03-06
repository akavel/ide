//! Example of the visualization JS wrapper API usage
// TODO remove once we have proper visualizations or replace with a nice d3 example.
// These implementations are neither efficient nor pretty, but get the idea across.

use crate::data;
use crate::component::visualization;



///////////////////////////////////////
// JavaScript builtin visualizations //
///////////////////////////////////////

/// Return a `JavaScript` Table visualization.
pub fn table_visualization() -> visualization::java_script::FallibleDefinition {
    let source = include_str!("java_script/table.js");

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

/// Return a `JavaScript` Scatter plot visualization.
pub fn scatter_plot_visualization() -> visualization::java_script::FallibleDefinition {
    let loading_scripts = include_str!("java_script/loading.js");
    let source          = include_str!("java_script/scatterPlot.js");
    let source          = format!("{}{}",loading_scripts,source);

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

/// Return a `JavaScript` Histogram visualization.
pub fn histogram_visualization() -> visualization::java_script::FallibleDefinition {
    let loading_scripts = include_str!("java_script/loading.js");
    let source          = include_str!("java_script/histogram.js");
    let source          = format!("{}{}",loading_scripts,source);

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

/// Return a `JavaScript` Map visualization.
pub fn geo_map_visualization() -> visualization::java_script::FallibleDefinition {
    let loading_scripts = include_str!("java_script/loading.js");
    let source          = include_str!("java_script/geoMap.js");
    let source          = format!("{}{}",loading_scripts,source);

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

/// Return a `JavaScript` Bubble visualization. This should not be used as it is a demo visualization.
pub fn bubble_visualization() -> visualization::java_script::FallibleDefinition {
    let source = include_str!("java_script/bubbleVisualization.js");

    visualization::java_script::Definition::new(data::builtin_library(),source)
}

/// Return an empty minimal `JavaScript` visualization. This should not be used except for testing.
pub fn empty_visualization() -> visualization::java_script::FallibleDefinition {
    let source = r#"
        class EmptyVisualization extends Visualization {}
        return EmptyVisualization;
    "#;

    visualization::java_script::Definition::new(data::builtin_library(),source)
}
