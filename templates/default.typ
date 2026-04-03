#set page(margin: 2.5cm)
#set text(font: "New Computer Modern", size: 11pt)
#set heading(numbering: none)
#set par(justify: true)

#let horizontalrule = line(length: 100%)


{% if title %}
#align(center)[
  #text(size: 18pt, weight: "bold")[ {{ title }} ]
]
#v(1em)
{% endif %}

$body$
