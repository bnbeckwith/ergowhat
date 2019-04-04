import { svg } from './ergoweb';
import { booted } from './ergoweb_bg';


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

function make_svg(src) {
  var fileContents = document.getElementById('keymap');
  fileContents.innerHTML = svg(src);
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

booted.then(() => {
  var km = getQueryVariable("fileurl");
    if(km) {
      select_url_tab();
      toggle_spinner();
      kmurl = decodeURIComponent(km);
      document.getElementById('urlname').value = kmurl;
      fetch(kmurl)
        .then(response => response.text() )
        .then(txt => make_svg(txt));
    }});
