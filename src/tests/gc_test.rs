#[cfg(test)]
use crate::{
    compile::get_builtins,
    expr::Expr,
    vm::{get_prelude, prepare_vm, step},
};

#[test]
fn gc_test() {
    let src = "
(define (queens board-size)
  (define (queen-cols k)
    (if (= k 0)
      (list empty-board)
      (filter
        (lambda (positions)
          (safe? k positions))
        (flatmap
          (lambda (rest-of-queens)
            (map (lambda (new-row)
                  (adjoin-position new-row k rest-of-queens))
              (enumerate-interval 1 board-size)))
          (queen-cols (- k 1))))))
  (queen-cols board-size))

(define
  (get-diagonals-backwards pos)
  (define row (car pos))
  (define col (car (cdr pos)))
  (flatmap
    (lambda (currCol)
      (list
        (list (- row currCol) (- col currCol))
        (list (+ row currCol) (- col currCol))))
    (enumerate-interval 0 col)))
(define (exists? pred coll)
  (cond
    ((null? coll) false)
    ((pred (car coll)) true)
    (else (exists? pred (cdr coll)))))

(define (on-diagonal pos new-queen)
  (exists? (lambda (diagonal-pos) (= diagonal-pos pos)) (get-diagonals-backwards new-queen)))

(assert (on-diagonal (list 4 4) (list 5 5)) true)
(assert (on-diagonal (list 4 3) (list 5 5)) false)
(assert (on-diagonal (list 1 1) (list 5 5)) true)

(define (safe? k all-positions)
  (define newQueen (car all-positions))
  (define positions (cdr all-positions))
  (define newQueenRow (car newQueen))
  (define newQueenCol (car (cdr newQueen)))
  (not
    (exists?
      (lambda (pos)
        (define posRow (car pos))
        (define posCol (car (cdr pos)))
        (or
          (= posCol newQueenCol)
          (or (= posRow newQueenRow)
            (on-diagonal pos newQueen))))
      positions)))

(define
  (adjoin-position row col rest-of-queens)
  (cons (list row col) rest-of-queens))

(define empty-board '())

(assert (length (queens 4)) 10)
        ";

    let globals = &get_builtins();
    let prelude = get_prelude();
    let (mut vm, _) = prelude
        .and_then(|env| {
            prepare_vm(
                &crate::parse::ParseInput {
                    source: src,
                    file_name: Some("gc_test"),
                },
                Some(env),
            )
        })
        .unwrap();
    let mut cycles_left = 100000 / 3;

    let all_envs = vm.envs.clone().keys().collect::<Vec<&String>>();
    fn find_envs(expr: &Expr, vec: &mut Vec<String>) -> () {
        match expr {
            Expr::Pair(l, r, ..) => {
                find_envs(l, vec);
                find_envs(r, vec);
            }
            Expr::Lambda(_, _, _, env) => {
                vec.push(env.clone());
            }
            _ => (),
        }
    }

    while cycles_left > 0 {
        match step(&mut vm, globals) {
            Err(err) => panic!("{err}"),
            Ok(()) if vm.callframes.is_empty() => {
                cycles_left = 0;
            }
            Ok(()) => {
                cycles_left -= 1;
            }
        }
    }

    let callframe_envs = vm
        .callframes
        .clone()
        .into_iter()
        .map(|x| x.env)
        .collect::<Vec<String>>();

    let lambda_refs = vm
        .callframes
        .clone()
        .into_iter()
        .map(|x| {
            let mut vec = vec![];
            x.chunk.code.into_iter().for_each(|instr| match instr {
                crate::vm::VMInstruction::Constant(expr) => find_envs(&expr, &mut vec),
                _ => {}
            });
            vec
        })
        .flatten()
        .collect::<Vec<String>>();
    let parent_env_refs = vm
        .envs
        .values()
        .map(|env| env.parent.clone())
        .flatten()
        .collect::<Vec<String>>();
    let mut expr_refs_in_envs = vec![];
    vm.envs.values().for_each(|env| {
        env.map
            .values()
            .for_each(|expr| find_envs(expr, &mut expr_refs_in_envs));
    });

    assert_eq!(vm.callframes.len(), 33);
    assert_eq!(callframe_envs.len(), 33);
    assert_eq!(parent_env_refs.len(), 3270);
    assert_eq!(expr_refs_in_envs.len(), 704);
    assert_eq!(lambda_refs.len(), 0);
    assert_eq!(cycles_left, 0);
    assert_eq!(3271, vm.envs.len());
}
