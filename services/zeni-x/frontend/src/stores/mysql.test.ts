import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useMySQLStore } from './mysql'

describe('MySQL Store - Query Limit Functions', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  describe('saveQueryLimit', () => {
    it('should save valid limit value', () => {
      const store = useMySQLStore()
      expect(store.saveQueryLimit(100)).toBe(true)
      expect(store.queryLimit).toBe(100)
      expect(localStorage.getItem('mysql-query-limit')).toBe('100')
    })

    it('should reject limit below 10', () => {
      const store = useMySQLStore()
      expect(store.saveQueryLimit(9)).toBe(false)
      expect(store.saveQueryLimit(0)).toBe(false)
      expect(store.saveQueryLimit(-1)).toBe(false)
    })

    it('should reject limit above 1000', () => {
      const store = useMySQLStore()
      expect(store.saveQueryLimit(1001)).toBe(false)
      expect(store.saveQueryLimit(5000)).toBe(false)
    })

    it('should accept boundary values', () => {
      const store = useMySQLStore()
      expect(store.saveQueryLimit(10)).toBe(true)
      expect(store.saveQueryLimit(1000)).toBe(true)
      expect(store.queryLimit).toBe(1000)
    })
  })

  describe('loadQueryLimit', () => {
    it('should load saved limit from localStorage', () => {
      localStorage.setItem('mysql-query-limit', '500')
      const store = useMySQLStore()
      store.loadQueryLimit()
      expect(store.queryLimit).toBe(500)
    })

    it('should use default value when localStorage is empty', () => {
      const store = useMySQLStore()
      store.loadQueryLimit()
      expect(store.queryLimit).toBe(100)
    })

    it('should ignore invalid values in localStorage', () => {
      localStorage.setItem('mysql-query-limit', 'invalid')
      const store = useMySQLStore()
      store.loadQueryLimit()
      expect(store.queryLimit).toBe(100)

      localStorage.setItem('mysql-query-limit', '5')
      store.loadQueryLimit()
      expect(store.queryLimit).toBe(100)
    })
  })

  describe('applyLimit', () => {
    beforeEach(() => {
      const store = useMySQLStore()
      store.saveQueryLimit(50)
    })

    it('should add LIMIT to simple SELECT', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users LIMIT 50')
    })

    it('should respect existing LIMIT', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users LIMIT 100'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users LIMIT 100')
    })

    it('should use custom limit when provided', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users'
      expect(store.applyLimit(sql, 200)).toBe('SELECT * FROM users LIMIT 200')
    })

    it('should remove trailing semicolon before adding LIMIT', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users;'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users LIMIT 50')
    })

    it('should not add LIMIT to non-SELECT statements', () => {
      const store = useMySQLStore()
      expect(store.applyLimit('INSERT INTO users VALUES (1, "test")'))
        .toBe('INSERT INTO users VALUES (1, "test")')
      expect(store.applyLimit('UPDATE users SET name = "test"'))
        .toBe('UPDATE users SET name = "test"')
      expect(store.applyLimit('DELETE FROM users WHERE id = 1'))
        .toBe('DELETE FROM users WHERE id = 1')
    })

    it('should handle SELECT with WHERE clause', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users WHERE id > 100'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users WHERE id > 100 LIMIT 50')
    })

    it('should handle SELECT with JOIN', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users u JOIN orders o ON u.id = o.user_id'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users u JOIN orders o ON u.id = o.user_id LIMIT 50')
    })

    it('should handle SELECT with ORDER BY', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users ORDER BY created_at DESC'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users ORDER BY created_at DESC LIMIT 50')
    })

    it('should not be fooled by LIMIT in string literals', () => {
      const store = useMySQLStore()
      const sql = `SELECT 'LIMIT' FROM users`
      expect(store.applyLimit(sql)).toBe(`SELECT 'LIMIT' FROM users LIMIT 50`)
    })

    it('should not be fooled by LIMIT in comments', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users -- LIMIT 1000'
      expect(store.applyLimit(sql)).toBe('SELECT * FROM users -- LIMIT 1000 LIMIT 50')
    })
  })

  describe('extractBaseQuery', () => {
    it('should remove LIMIT clause', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users LIMIT 100'
      expect(store.extractBaseQuery(sql)).toBe('SELECT * FROM users')
    })

    it('should remove LIMIT with OFFSET', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users LIMIT 100 OFFSET 200'
      expect(store.extractBaseQuery(sql)).toBe('SELECT * FROM users')
    })

    it('should remove OFFSET with LIMIT', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users OFFSET 200 LIMIT 100'
      expect(store.extractBaseQuery(sql)).toBe('SELECT * FROM users')
    })

    it('should remove trailing semicolon', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users LIMIT 100;'
      expect(store.extractBaseQuery(sql)).toBe('SELECT * FROM users')
    })

    it('should not modify queries without LIMIT/OFFSET', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users WHERE id > 100'
      expect(store.extractBaseQuery(sql)).toBe('SELECT * FROM users WHERE id > 100')
    })

    it('should handle complex queries', () => {
      const store = useMySQLStore()
      const sql = 'SELECT * FROM users WHERE active = 1 ORDER BY created_at DESC LIMIT 50 OFFSET 100'
      expect(store.extractBaseQuery(sql))
        .toBe('SELECT * FROM users WHERE active = 1 ORDER BY created_at DESC')
    })
  })
})
