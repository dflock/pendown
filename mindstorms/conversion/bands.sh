# Prints the bands of non-blank rows in a page image: bands.sh image gap
convert "$1" -colorspace gray -threshold 70% -negate -resize 1x$(identify -format %h "$1")! txt:- | awk -F'[,: ()]+' 'NR>1{if ($4>0) print $2}' | awk -v g=${2:-4} 'NR==1{s=$1;p=$1;next} {if ($1>p+g) {printf "%d-%d ", s, p; s=$1} p=$1} END{print s"-"p}'
