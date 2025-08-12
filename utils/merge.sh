#!/bin/bash

# =============================================================================
# Script Name: merge.sh
# Description: Merge WASM and JS files into an HTML template
# 
# Usage: ./merge.sh [OPTIONS] <wasm_file> <js_file>
# 
# Options:
#   -h, --help      Display this help message
#   -v, --verbose   Enable verbose output
#   -d, --dry-run   Perform a dry run without making changes
#   -o, --output    Output HTML file (default: output.html)
#   -t, --template  Custom HTML template file
# 
# Arguments:
#   wasm_file       Path to the WASM file to convert to base64
#   js_file         Path to the JavaScript file to include
# 
# Examples:
#   ./merge.sh app.wasm app.js
#   ./merge.sh -v -o index.html app.wasm app.js
#   ./merge.sh --template custom.html app.wasm app.js
# 
# Exit Codes:
#   0 - Success
#   1 - General error
#   2 - Invalid arguments
#   3 - WASM file not found
#   4 - JS file not found
#   5 - Template file not found
#   6 - Output directory not writable
# 
# Dependencies:
#   - bash 4.0+
#   - base64 (coreutils)
#   - cat
# 
# =============================================================================

# =============================================================================
# Configuration
# =============================================================================
SCRIPT_NAME=$(basename "$0")
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default values
VERBOSE=false
DRY_RUN=false
WASM_FILE=""
JS_FILE=""
OUTPUT_FILE="output.html"
TEMPLATE_FILE=""

# =============================================================================
# Functions
# =============================================================================

# Print usage information
usage() {
    cat << EOF
Usage: $SCRIPT_NAME [OPTIONS] <wasm_file> <js_file>

Options:
    -h, --help      Display this help message
    -v, --verbose   Enable verbose output
    -d, --dry-run   Perform a dry run without making changes
    -o, --output    Output HTML file (default: output.html)
    -t, --template  Custom HTML template file

Arguments:
    wasm_file       Path to the WASM file to convert to base64
    js_file         Path to the JavaScript file to include

Examples:
    $SCRIPT_NAME app.wasm app.js
    $SCRIPT_NAME -v -o index.html app.wasm app.js
    $SCRIPT_NAME --template custom.html app.wasm app.js

Exit Codes:
    0 - Success
    1 - General error
    2 - Invalid arguments
    3 - WASM file not found
    4 - JS file not found
    5 - Template file not found
    6 - Output directory not writable

EOF
}

# Print error message to stderr
error() {
    echo "ERROR: $*" >&2
}

# Print warning message to stderr
warning() {
    echo "WARNING: $*" >&2
}

# Print info message (only if verbose is enabled)
info() {
    if [[ "$VERBOSE" == true ]]; then
        echo "INFO: $*"
    fi
}

# Print debug message (only if verbose is enabled)
debug() {
    if [[ "$VERBOSE" == true ]]; then
        echo "DEBUG: $*"
    fi
}

# Validate arguments
validate_args() {
    if [[ -z "$WASM_FILE" ]]; then
        error "WASM file argument is required"
        usage
        exit 2
    fi
    
    if [[ -z "$JS_FILE" ]]; then
        error "JavaScript file argument is required"
        usage
        exit 2
    fi
    
    if [[ ! -f "$WASM_FILE" ]]; then
        error "WASM file '$WASM_FILE' does not exist"
        exit 3
    fi
    
    if [[ ! -f "$JS_FILE" ]]; then
        error "JavaScript file '$JS_FILE' does not exist"
        exit 4
    fi
    
    # Check if output directory is writable
    local output_dir=$(dirname "$OUTPUT_FILE")
    if [[ "$output_dir" != "." ]] && [[ ! -w "$output_dir" ]]; then
        error "Output directory '$output_dir' is not writable"
        exit 6
    fi
    
    # Check template file if specified
    if [[ -n "$TEMPLATE_FILE" ]] && [[ ! -f "$TEMPLATE_FILE" ]]; then
        error "Template file '$TEMPLATE_FILE' does not exist"
        exit 5
    fi
}

# Convert WASM file to base64
convert_wasm_to_base64() {
    info "Converting WASM file to base64: $WASM_FILE"
    
    # Convert WASM to base64 and store in variable
    WASM_BASE64=$(base64 -w 0 "$WASM_FILE")
    if [[ $? -ne 0 ]]; then
        error "Failed to convert WASM file to base64"
        exit 1
    fi
    
    info "WASM file converted to base64 successfully"
    debug "Base64 length: ${#WASM_BASE64} characters"
}

# Read JavaScript file content
read_js_file() {
    info "Reading JavaScript file: $JS_FILE"
    
    # Read JS file content
    JS_CONTENT=$(cat "$JS_FILE")
    if [[ $? -ne 0 ]]; then
        error "Failed to read JavaScript file"
        exit 1
    fi
    
    info "JavaScript file read successfully"
    debug "JavaScript content length: ${#JS_CONTENT} characters"
}

# Generate default HTML template
generate_default_template() {
    cat << 'EOF'
<!DOCTYPE html>
<html lang="en">

<head>
  <meta content="text/html;charset=utf-8" http-equiv="Content-Type" />
  <style type="text/css">
    * {
      box-sizing: border-box;
    }

    body {
      margin: 0;
    }

    canvas {
      width: 100%;
      height: 100%;
      position: fixed;
    }

    #ui-overlay {
      position: fixed;
      width: 100%;
    }

    #ui-overlay input[type="range"] {
      width: 100%;
      margin: 0;
      padding: 5px;
    }
  </style>
</head>

<body>
  <noscript>
    This interactive example cannot run without JavaScript. Sorry.
  </noscript>
  <canvas id="canvas">
    This interactive example cannot run without canvas support. Sorry.
  </canvas>
  <div id="ui-overlay">
    <span id="ui-message"></span>
  </div>
  <script>
    const htmlCanvas = document.getElementById('canvas');
    function resizeCanvas() {
      htmlCanvas.width = window.innerWidth * window.devicePixelRatio;
      htmlCanvas.height = window.innerHeight * window.devicePixelRatio;
    }
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas, false);
  </script>
  <script type="module">

    JS_CONTENT_PLACEHOLDER

    const ui = document.getElementById('ui-message');
    ui.innerText = 'Executing wasm...';

    (async () => {
      const data = "data:application/wasm;base64,WASM_BASE64_PLACEHOLDER";
      try {
        if( typeof(init) !== 'undefined' ) {
          await init(data);
        } else {
          await wasm_bindgen(data);
        }
        ui.innerText = '';
      } catch(err) {
        ui.innerText = `Panic! ${err}. Check console log for details.`;
      }
    })();
  </script>
</body>

</html>
EOF
}

# Generate HTML output
generate_html() {
    info "Generating HTML output: $OUTPUT_FILE"
    
    # Generate HTML content
    if [[ -n "$TEMPLATE_FILE" ]]; then
        info "Using custom template: $TEMPLATE_FILE"
        HTML_CONTENT=$(cat "$TEMPLATE_FILE")
    else
        info "Using default template"
        HTML_CONTENT=$(generate_default_template)
    fi
    
    # Replace placeholders with actual content
    HTML_CONTENT="${HTML_CONTENT//WASM_BASE64_PLACEHOLDER/$WASM_BASE64}"
    HTML_CONTENT="${HTML_CONTENT//JS_CONTENT_PLACEHOLDER/"$JS_CONTENT"}"

    if [[ "$DRY_RUN" == true ]]; then
        info "DRY RUN MODE - No changes will be made"
        return 0
    fi

    # Write to output file
    echo "$HTML_CONTENT" > "$OUTPUT_FILE"
    if [[ $? -ne 0 ]]; then
        error "Failed to write output file: $OUTPUT_FILE"
        exit 1
    fi
    
    info "HTML file generated successfully: $OUTPUT_FILE"
    info "File size: $(wc -c < "$OUTPUT_FILE") bytes"
}

# Main merge function
perform_merge() {
    info "Starting WASM/JS merge operation"
    info "WASM file: $WASM_FILE"
    info "JavaScript file: $JS_FILE"
    info "Output file: $OUTPUT_FILE"
        
    convert_wasm_to_base64
    read_js_file
    generate_html
    
    info "Merge completed successfully"
}

# =============================================================================
# Main Script
# =============================================================================

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -t|--template)
            TEMPLATE_FILE="$2"
            shift 2
            ;;
        -*)
            error "Unknown option: $1"
            usage
            exit 2
            ;;
        *)
            if [[ -z "$WASM_FILE" ]]; then
                WASM_FILE="$1"
            elif [[ -z "$JS_FILE" ]]; then
                JS_FILE="$1"
            else
                error "Too many arguments"
                usage
                exit 2
            fi
            shift
            ;;
    esac
done

# Main execution
main() {
    info "Script started: $SCRIPT_NAME"
    info "Script directory: $SCRIPT_DIR"
    
    validate_args
    perform_merge
    
    info "Script completed successfully"
    echo "✅ Successfully generated: $OUTPUT_FILE"
    exit 0
}

# Trap errors and cleanup
trap 'error "Script interrupted"; exit 1' INT TERM

# Run main function
main "$@"
