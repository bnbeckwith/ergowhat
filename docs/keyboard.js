function layer_click(layer) {
  alert("Layer " + layer);
}

function onlylayer(layer){
  var l;
  for (l=0; l<32; l++){
    var elem = document.getElementById("layer" + l);
    if (elem) {
      if (l == layer) {
        elem.setAttribute('visibility','visible');
      }else{
        elem.setAttribute('visibility','hidden');
      }
    }
  }
}

function layer0on(layer) {
  layeron(0);
}

function layeron(layer) {
  document.getElementById("layer" + layer).setAttribute('visibility','visible')
}

function layeroff(layer) {
  document.getElementById("layer" + layer).setAttribute('visibility','hidden')
}

window.onload = function(){
  document.getElementById("layer0").setAttribute('visibility','visible');  
}
