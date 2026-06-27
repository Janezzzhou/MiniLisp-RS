(define (insert x xs)
  (cond
    ((null? xs) (list x))
    ((<= x (car xs)) (cons x xs))
    (else (cons (car xs) (insert x (cdr xs))))))

(define (sort xs)
  (if (null? xs)
      '()
      (insert (car xs) (sort (cdr xs)))))

(print (sort '(12 71 2 15 29 82 87 8 18 66 81 25 63 97 40 3 93 58 53 31 47)))
