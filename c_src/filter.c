// author https://github.com/MIrrox27/rkn-simulator
// c_src/filter.rs

// Здесь будет происходить обработка доменов сайтов, основная логика заключается в том что:
// Если сайта нет в черном списке, возвращается 0
// Если сайт есть в черном списке вернем 1

/*  Понадобятся функции для:

  - Записи нового домена в черный список 
  - Удаления домена из черного списка
  - Получения домена из целого URL (дано: "google.com/search=123" => вернуть: "google.com")
  - Проверки есть ли домен в черном списке (основная функция этого файла)
*/ 



#include <stdio.h>
#include <stdlib.h>
#include <string.h>


typedef struct DomensBlackList { // Своеобразный связанный список 
  char str[64]; // Поле для ввода 1 домена
  struct DomensBlackList *next; // Указатель на сл элемент списка
  } DomensBlackList;


  DomensBlackList *global_list = NULL; // Указатель на первый элемент массива 















