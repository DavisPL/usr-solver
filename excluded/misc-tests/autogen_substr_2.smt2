(declare-const s1 String)
(declare-const y String)

;(assert (= "oz72" (str.substr s1 14 4)))
;(assert (= "mmlz" (str.substr s1 6 4)))
;(assert (= "0792" (str.substr s1 7 4)))

(assert
    (str.in_re y
        (re.++
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "oz72") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "oz72")
                        (re.++ re.allchar re.allchar re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "oz72"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "oz72")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "mmlz") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "mmlz")
                        (re.++ re.allchar re.allchar re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "mmlz"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "mmlz")
                        (str.to_re "")
                    )
                )
            )
            (re.union
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "0792") (re.* re.allchar))
                    )
                    (re.inter
                        (str.to_re "0792")
                        (re.++ re.allchar re.allchar re.allchar re.allchar)
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (str.to_re "0792"))
                    )
                    (re.inter
                        (str.to_re s1)
                        (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar (re.* re.allchar))))
                    )
                )
                (re.++
                    (re.inter
                        (str.to_re s1)
                        (re.comp (re.++ re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar re.allchar (re.* re.allchar)))
                    )
                    (re.inter
                        (str.to_re "0792")
                        (str.to_re "")
                    )
                )
            )
        )
    )
)
(check-sat)
