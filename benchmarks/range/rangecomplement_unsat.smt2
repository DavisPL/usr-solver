(declare-fun x () String)

(assert (and
  (str.in_re x (re.range "a" "c")  )
  (not (str.in_re x (str.to_re "a"))) 
  (not (str.in_re x (str.to_re "b"))) 
  (not (str.in_re x (str.to_re "c"))) 
))

(check-sat)
