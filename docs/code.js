// Copied from
// https://css-tricks.com/snippets/javascript/get-url-variables/
// Thanks!
function getQueryVariable(variable)
{
       var query = window.location.search.substring(1);
       var vars = query.split("&");
       for (var i=0;i<vars.length;i++) {
               var pair = vars[i].split("=");
               if(pair[0] == variable){return pair[1];}
       }
       return(false);
}

function fetchAndInstantiate(url, importObject) {
  return fetch(url).then(response =>
    response.arrayBuffer()
  ).then(bytes =>
    WebAssembly.instantiate(bytes, importObject)
  ).then(results =>
    results.instance
  );
}

// Copy a nul-terminated string from the buffer pointed to.
// Consumes the old data and thus deallocated it.
function copyCStr(module, ptr) {
  let orig_ptr = ptr;
  const collectCString = function* () {
    let memory = new Uint8Array(module.memory.buffer);
    while (memory[ptr] !== 0) {
      if (memory[ptr] === undefined) { throw new Error("Tried to read undef mem") }
      yield memory[ptr]
      ptr += 1
    }
  }

  const buffer_as_u8 = new Uint8Array(collectCString())
  const utf8Decoder = new TextDecoder("UTF-8");
  const buffer_as_utf8 = utf8Decoder.decode(buffer_as_u8);
  module.dealloc_str(orig_ptr);
  return buffer_as_utf8
}

function getStr(module, ptr, len) {
  const getData = function* (ptr, len) {
    let memory = new Uint8Array(module.memory.buffer);
    for (let index = 0; index < len; index++) {
      if (memory[ptr] === undefined) { throw new Error(`Tried to read undef mem at ${ptr}`) }
      yield memory[ptr + index]
    }
  }

  const buffer_as_u8 = new Uint8Array(getData(ptr/8, len/8));
  const utf8Decoder = new TextDecoder("UTF-8");
  const buffer_as_utf8 = utf8Decoder.decode(buffer_as_u8);
  return buffer_as_utf8;
}

function newString(module, str) {
  const utf8Encoder = new TextEncoder("UTF-8");
  let string_buffer = utf8Encoder.encode(str)
  let len = string_buffer.length
  let ptr = module.alloc(len+1)

  let memory = new Uint8Array(module.memory.buffer);
  for (i = 0; i < len; i++) {
    memory[ptr+i] = string_buffer[i]
  }

  memory[ptr+len] = 0;

  return ptr;
}

window.Module = {};

var Ergodox = {
  svg: function(str){
    let buf = newString(Module, str);
    let outptr = Module.svg(buf);
    let result = copyCStr(Module, outptr);
    return result;
  }
};

function make_svg(src) {
  var fileContents = document.getElementById('keymap');
  fileContents.innerHTML = Ergodox.svg(src);
  layer0on();
  toggle_spinner();
}

function toggle_spinner() {
  var spinner = document.getElementById("spinny");
  if (spinner.style.display == "block") {
    spinner.style.display = "none";
  }else {
    spinner.style.display = "block";
  }
}

function select_url_tab(){
  var utab = document.getElementById("urltab");
  utab.checked = true;
}

fetchAndInstantiate("ergoweb.wasm", {})
  .then(mod => {
    Module.alloc       = mod.exports.alloc;
    Module.dealloc     = mod.exports.dealloc;
    Module.dealloc_str = mod.exports.dealloc_str;
    Module.memory      = mod.exports.memory;
    Module.svg         = mod.exports.svg;
  })
  .then(function() {
    var km = getQueryVariable("fileurl");
    if(km) {
      select_url_tab();
      toggle_spinner();
      kmurl = decodeURIComponent(km);
      document.getElementById('urlname').value = kmurl;
      fetch(kmurl)
        .then(response => response.text() )
        .then(txt => make_svg(txt));
    }
  });
