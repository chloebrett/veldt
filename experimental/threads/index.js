const canvas = document.getElementById('canvas');

console.log("Loaded JS!");

// First up, but try to do feature detection to provide better error messages
function loadWasm() {
  let msg = 'This demo requires a recent-ish browser (Firefox/Chrome for the last few years)'
  if (typeof SharedArrayBuffer !== 'function') {
    alert('this browser does not have SharedArrayBuffer support enabled' + '\n\n' + msg);
    return
  }
  // Test for bulk memory operations with passive data segments
  //  (module (memory 1) (data passive ""))
  const buf = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x05, 0x03, 0x01, 0x00, 0x01, 0x0b, 0x03, 0x01, 0x01, 0x00]);
  if (!WebAssembly.validate(buf)) {
    alert('this browser does not support passive Wasm memory, demo does not work' + '\n\n' + msg);
    return
  }

  console.log("Loading wasm!");

  wasm_bindgen().then(run).catch(console.error);
}

loadWasm();

const { WebHandle, test_threads } = wasm_bindgen;

function run() {
  console.log("Starting!");

  //test_threads();

  /*console.log(WebHandle);
  const handle = new WebHandle();
  console.log(handle);
  handle.start();
  console.log("Started");*/

}
