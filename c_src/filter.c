// author https://github.com/MIrrox27/rkn-simulator
// c_src/filter.c

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



void append_to_blacklist(const char *text){
  DomensBlackList *new_node = (DomensBlackList*) malloc(sizeof(DomensBlackList));
  if (new_node == NULL) return; 

  strncpy(new_node->str, text, sizeof(new_node->str) - 1); // Копируем строку с ограничением длины
  new_node->str[sizeof(new_node->str) - 1] = '\0'; // Безопасное завершение строки

  new_node->next = global_list;
  global_list = new_node;
}


void delete_from_blacklist(const char *text){
  DomensBlackList *current = global_list;

  while (current != NULL){
    DomensBlackList *temp = current;

    if (strcmp(current->str, text)){
      free(temp);
    } 
    current = current->next;
  } 
}













