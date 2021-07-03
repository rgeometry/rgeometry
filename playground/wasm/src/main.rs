use std::fs;

fn main() -> std::io::Result<()> {
    println!("Packing wasm into single HTML file");
    println!("Javascript input: <- pkg/wasm.js");
    let javascript = fs::read_to_string("pkg/wasm.js")?;
    println!("Wasm module:      <- pkg/wasm_bg.wasm");
    let module = fs::read("pkg/wasm_bg.wasm")?;
    let encoded = base64::encode(module);

    println!("Html result:      -> rgeometry-wasm.html");
    fs::write("rgeometry-wasm.html", HTML_TEMPLATE
        .replace("JAVASCRIPT", &javascript)
        .replace("WASM_MODULE", &encoded)
    )?;
    Ok(())
}

static HTML_TEMPLATE: &'static str = r###"
<html>

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


    JAVASCRIPT

    const ui = document.getElementById('ui-message');
    ui.innerText = 'Loading...';

    async function run() {
      const data = "data:application/wasm;base64,WASM_MODULE";
      // Support both --target=web and --target=no-modules
      if( typeof(init) !== 'undefined' ) {
        await init(data);
      } else {
        await wasm_bindgen(data);
      }
      ui.innerText = '';
    }

    run();
  </script>
</body>

</html>
"###;
